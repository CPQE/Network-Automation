/*
switch_collector.cpp

Connects to a Cisco Catalyst switch over SSH, runs a small set of "show" commands,
prints raw command output to stdout, and prints concise parsed summaries for each
command.

Notes:
 - Uses libssh (https://www.libssh.org). Install libssh and the dev headers for your platform.
 - Compile:
     g++ -std=c++17 switch_collector.cpp -lssh -o switch_collector
 - Run:
     ./switch_collector --host 10.0.0.10 --user neteng --pass secret

This is a compact, single-file demo intended for lab/useful-starting-point purposes.
Do NOT use plaintext passwords in production; use SSH keys and a secrets store.
*/

#include <libssh/libssh.h>
#include <iostream>
#include <string>
#include <vector>
#include <regex>
#include <sstream>
#include <cstdlib>
#include <cstring>
#include <map>

using std::string;
using std::vector;
using std::cout;
using std::cerr;
using std::endl;

// -----------------------------
// SSH helper
// -----------------------------
static string run_ssh_command(ssh_session session, const string &cmd, int timeout_seconds = 10) {
    ssh_channel channel = ssh_channel_new(session);
    if (!channel) {
        throw std::runtime_error("Failed to create SSH channel");
    }
    int rc = ssh_channel_open_session(channel);
    if (rc != SSH_OK) {
        ssh_channel_free(channel);
        throw std::runtime_error("Failed to open SSH channel: " + string(ssh_get_error(session)));
    }

    // Request a PTY and allocate terminal width if needed so some devices paginate differently.
    // Many IOS devices honor "terminal length 0" instead of PTY sizing; we will send that as first command if necessary.
    rc = ssh_channel_request_exec(channel, cmd.c_str());
    if (rc != SSH_OK) {
        ssh_channel_close(channel);
        ssh_channel_free(channel);
        throw std::runtime_error("Failed to execute command: " + cmd + " : " + string(ssh_get_error(session)));
    }

    std::string output;
    char buffer[4096];
    int nbytes;
    // read until EOF
    while ((nbytes = ssh_channel_read_timeout(channel, buffer, sizeof(buffer), 0, timeout_seconds * 1000)) > 0) {
        output.append(buffer, buffer + nbytes);
    }

    ssh_channel_send_eof(channel);
    ssh_channel_close(channel);
    ssh_channel_free(channel);
    return output;
}

// Convenience wrapper: open session, authenticate, return session
static ssh_session open_ssh_session(const string &host, int port, const string &user, const string &pass, int timeout_seconds = 10) {
    ssh_session session = ssh_new();
    if (!session) {
        throw std::runtime_error("Failed to create SSH session object");
    }
    ssh_options_set(session, SSH_OPTIONS_HOST, host.c_str());
    ssh_options_set(session, SSH_OPTIONS_PORT, &port);
    ssh_options_set(session, SSH_OPTIONS_USER, user.c_str());
    // set timeout
    ssh_options_set(session, SSH_OPTIONS_TIMEOUT, &timeout_seconds);

    int rc = ssh_connect(session);
    if (rc != SSH_OK) {
        string err = ssh_get_error(session);
        ssh_free(session);
        throw std::runtime_error("Error connecting to " + host + ": " + err);
    }

    // Simple password auth (demo). In production prefer publickey auth.
    rc = ssh_userauth_password(session, NULL, pass.c_str());
    if (rc != SSH_AUTH_SUCCESS) {
        string err = ssh_get_error(session);
        ssh_disconnect(session);
        ssh_free(session);
        throw std::runtime_error("Authentication failed: " + err);
    }
    return session;
}

// -----------------------------
// Small text utilities
// -----------------------------
static vector<string> split_lines(const string &s) {
    vector<string> out;
    std::istringstream iss(s);
    string line;
    while (std::getline(iss, line)) out.push_back(line);
    return out;
}

static string trim(const string &s) {
    auto start = s.find_first_not_of(" \t\r\n");
    if (start == string::npos) return "";
    auto end = s.find_last_not_of(" \t\r\n");
    return s.substr(start, end - start + 1);
}

static vector<string> split_by_regex(const string &s, const std::regex &re) {
    vector<string> out;
    std::sregex_token_iterator it(s.begin(), s.end(), re, -1);
    std::sregex_token_iterator end;
    for (; it != end; ++it) {
        out.push_back(it->str());
    }
    return out;
}

// -----------------------------
// Parsers (concise, best-effort)
// -----------------------------
struct ShowVersionInfo {
    string hostname;
    string uptime;
    string os_version;
    string model;
    string serial;
    string system_image;
};

static ShowVersionInfo parse_show_version(const string &text) {
    ShowVersionInfo info;
    auto lines = split_lines(text);
    for (const auto &raw : lines) {
        string line = trim(raw);
        if (line.find(" uptime is ") != string::npos && info.hostname.empty()) {
            // Router1 uptime is ...
            std::istringstream iss(line);
            iss >> info.hostname;
            auto pos = line.find(" uptime is ");
            if (pos != string::npos) info.uptime = trim(line.substr(pos + 11));
        }
        if (line.find("Cisco IOS") != string::npos || (line.find("Version") != string::npos && line.find("IOS") != string::npos)) {
            if (info.os_version.empty()) info.os_version = line;
        }
        std::smatch m;
        if (std::regex_search(line, m, std::regex(R"(Model number\s*:\s*(\S+))", std::regex::icase))) {
            info.model = m[1];
        }
        if (std::regex_search(line, m, std::regex(R"(Processor board ID\s*:?\s*(\S+))", std::regex::icase))) {
            info.serial = m[1];
        }
        if (line.find("System image file is") != string::npos) {
            auto pos = line.find("System image file is");
            info.system_image = trim(line.substr(pos + 21));
            // remove quotes
            if (!info.system_image.empty() && info.system_image.front() == '"') info.system_image.erase(0,1);
            if (!info.system_image.empty() && info.system_image.back() == '"') info.system_image.pop_back();
        }
    }
    return info;
}

struct InterfaceBrief { string name, ip, ok, method, status, protocol; };
static vector<InterfaceBrief> parse_ip_interface_brief(const string &text) {
    vector<InterfaceBrief> res;
    auto lines = split_lines(text);
    bool header = false;
    for (auto &l

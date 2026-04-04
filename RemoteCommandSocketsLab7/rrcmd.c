#include <sys/socket.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <pthread.h>
#include <time.h>

#define BUF_SIZE 4096
#define LENGTH_PREFIX 10
#define MAX_THREADS 10

void error(char *msg) {
    fprintf(stderr, "%s: %s\n", msg, strerror(errno));
    exit(1);
}

// Sends exactly n bytes to socket
int send_all(int sock, char *buf, int len) {
    int total = 0;
    while (total < len) {
        int s = send(sock, buf + total, len - total, 0);
        if (s == -1) return -1;
        total += s;
    }
    return total;
}

// Gets current time as a formatted string into buf
void get_timestamp(char *buf, int len) {
    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    strftime(buf, len, "%Y-%m-%d %H:%M:%S", t);
}

// Runs a command using popen() and returns output in a heap allocated string
// Caller is responsible for freeing the returned string
char *run_command(char *command) {
    FILE *fp = popen(command, "r");
    if (!fp) {
        char *err = strdup("Failed to run command\n");
        return err;
    }

    char *output = malloc(BUF_SIZE);
    if (!output) error("malloc failed");
    output[0] = '\0';

    size_t total = 0;
    char line[256];
    while (fgets(line, sizeof(line), fp) != NULL) {
        size_t linelen = strlen(line);
        if (total + linelen >= BUF_SIZE - 1) break;
        strcat(output, line);
        total += linelen;
    }

    pclose(fp);
    return output;
}

// Reads a newline terminated message from the socket into buf
int read_line(int sock, char *buf, int len) {
    int total = 0;
    char c;
    while (total < len - 1) {
        int r = recv(sock, &c, 1, 0);
        if (r <= 0) break;
        buf[total++] = c;
        if (c == '\n') break;
    }
    buf[total] = '\0';
    return total;
}

// Sends a result back to client with 10 byte length prefix
int send_result(int sock, char *timestamp, char *output) {
    // Build full response: timestamp + output
    char response[BUF_SIZE];
    snprintf(response, sizeof(response), "[%s]\n%s", timestamp, output);

    int response_len = strlen(response);

    // Build 10 byte length prefix, zero padded
    char prefix[LENGTH_PREFIX + 1];
    snprintf(prefix, sizeof(prefix), "%010d", response_len);

    // Send prefix then response
    if (send_all(sock, prefix, LENGTH_PREFIX) == -1) return -1;
    if (send_all(sock, response, response_len) == -1) return -1;
    return 0;
}

// Struct to pass data into each thread
typedef struct {
    int connect_d;
    struct sockaddr_storage client_addr;
} ClientArgs;

// Thread function — handles one client connection
void *handle_client(void *arg) {
    ClientArgs *args = (ClientArgs *)arg;
    int connect_d = args->connect_d;

    // Get client IP for logging
    char client_ip[INET6_ADDRSTRLEN];
    struct sockaddr_storage *addr = &args->client_addr;
    if (addr->ss_family == AF_INET) {
        inet_ntop(AF_INET, &((struct sockaddr_in *)addr)->sin_addr,
                  client_ip, sizeof(client_ip));
    } else {
        inet_ntop(AF_INET6, &((struct sockaddr_in6 *)addr)->sin6_addr,
                  client_ip, sizeof(client_ip));
    }

    char timestamp[64];
    get_timestamp(timestamp, sizeof(timestamp));
    printf("[%s] Connection from %s\n", timestamp, client_ip);

    // Read the message from client
    char buf[BUF_SIZE];
    if (read_line(connect_d, buf, sizeof(buf)) <= 0) {
        fprintf(stderr, "Failed to read from client %s\n", client_ip);
        close(connect_d);
        free(args);
        return NULL;
    }

    // Parse: "execution_count time_delay command"
    int execution_count, time_delay;
    char command[BUF_SIZE];
    if (sscanf(buf, "%d %d %[^\n]", &execution_count, &time_delay, command) != 3) {
        fprintf(stderr, "Malformed message from %s: %s\n", client_ip, buf);
        close(connect_d);
        free(args);
        return NULL;
    }

    printf("[%s] Client %s requested: '%s' x%d every %ds\n",
           timestamp, client_ip, command, execution_count, time_delay);

    // Execute command execution_count times with time_delay between each
    for (int i = 0; i < execution_count; i++) {
        get_timestamp(timestamp, sizeof(timestamp));

        char *output = run_command(command);
        if (send_result(connect_d, timestamp, output) == -1) {
            fprintf(stderr, "Failed to send result to %s\n", client_ip);
            free(output);
            break;
        }
        free(output);

        // Sleep between executions, but not after the last one
        if (i < execution_count - 1)
            sleep(time_delay);
    }

    get_timestamp(timestamp, sizeof(timestamp));
    printf("[%s] Connection from %s closed\n", timestamp, client_ip);

    close(connect_d);
    free(args);
    return NULL;
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: rcmdd <port>\n");
        exit(1);
    }

    int port = atoi(argv[1]);

    // Create TCP listener socket
    int listener_d = socket(PF_INET6, SOCK_STREAM, 0);
    if (listener_d == -1) error("Can't open socket");

    // Allow reuse of port immediately after server exits
    int reuse = 1;
    if (setsockopt(listener_d, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse)) == -1)
        error("Can't set SO_REUSEADDR");

    // Allow both IPv4 and IPv6 connections
    int ipv6only = 0;
    if (setsockopt(listener_d, IPPROTO_IPV6, IPV6_V6ONLY, &ipv6only, sizeof(ipv6only)) == -1)
        error("Can't set IPV6_V6ONLY");

    // Bind to all interfaces on given port
    struct sockaddr_in6 name;
    memset(&name, 0, sizeof(name));
    name.sin6_family = AF_INET6;
    name.sin6_port = htons(port);
    name.sin6_addr = in6addr_any;

    if (bind(listener_d, (struct sockaddr *)&name, sizeof(name)) == -1)
        error("Can't bind to port");

    if (listen(listener_d, 10) == -1)
        error("Can't listen");

    printf("Server listening on port %d\n", port);

    // Accept loop — spawn a thread per connection
    while (1) {
        ClientArgs *args = malloc(sizeof(ClientArgs));
        if (!args) error("malloc failed");

        socklen_t address_size = sizeof(args->client_addr);
        args->connect_d = accept(listener_d,
                                 (struct sockaddr *)&args->client_addr,
                                 &address_size);
        if (args->connect_d == -1) {
            free(args);
            fprintf(stderr, "Accept failed: %s\n", strerror(errno));
            continue;
        }

        // Spawn thread to handle this client
        pthread_t thread;
        if (pthread_create(&thread, NULL, handle_client, (void *)args) != 0) {
            fprintf(stderr, "Failed to create thread: %s\n", strerror(errno));
            close(args->connect_d);
            free(args);
            continue;
        }

        // Detach so thread cleans up automatically when done
        pthread_detach(thread);
    }

    close(listener_d);
    return 0;
}

/*
`pthread_detach` is used instead of `pthread_join` — since the server runs forever and we don't need to wait for threads to finish, detaching lets the thread clean up its own resources automatically when it exits. `pthread_join` would block the main thread waiting for each client to finish which would defeat the concurrency.

`popen` is cleaner than `exec` here because it gives you a `FILE*` to read from directly — no temp files, no pipe setup, no `fork` needed. The tradeoff is it uses `/bin/sh` to run the command which is fine for this assignment.

`IPV6_V6ONLY` set to 0 means the server accepts both IPv4 and IPv6 connections on the ssame socket, same idea as your Rust code using `[::]`.

The `ClientArgs` struct passed via `malloc` into each thread is important — if you passed a stack variable it could be overwritten by the next `accept` call before the thread reads it.

Compile with:
```
gcc -o rcmdd rcmdd.c -lpthread
gcc -o rcmd rcmd.c */
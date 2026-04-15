Only ansible works since FABRIC doesn't let you have admin access to install teraform as a package on the machines. 
Must use the python scripts and then run ansible: 
cd Ansible
ansible-playbook -i hosts.ini site.yml \
  --ssh-extra-args "-F /home/fabric/work/fabric_config/ssh_config"
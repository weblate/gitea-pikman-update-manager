#! /bin/bash

mkdir -p ~/.ssh
ssh-keyscan  -H pika-os.com >> ~/.ssh/known_hosts
ssh-keyscan  -H git.pika-os.com >> ~/.ssh/known_hosts
#echo -e "Host git.pika-os.com\n\tStrictHostKeyChecking no\n" >> ~/.ssh/config
#echo -e "Host git.pika-os.com\n\tIdentityFile ~/.ssh/id_rsa\n\tStrictHostKeyChecking no\n" > ~/.ssh/config
#ssh-agent -a $SSH_AUTH_SOCK > /dev/null
#ssh-add - <<< "$1"

#export GIT_SSH_COMMAND="ssh -F ~/.ssh/config"

#ssh -o StrictHostKeyChecking=no -vT git@git.pika-os.com
#ssh -vT git@git.pika-os.com

# Commit changes to git
git config --global user.name 'Github Gitea Push Back Key - Cosmo'
git config --global user.email 'cosmo@pika-os.com'
#git config --global --add safe.directory /__w/gitea-pikman-update-manager/gitea-pikman-update-manager

git clone git@git.pika-os.com:custom-gui-packages/pikman-update-manager

rm -rfv ./gitea-pikman-update-manager/.git
cp -rfv ./pikman-update-manager/.git ./gitea-pikman-update-manager/

cd ./gitea-pikman-update-manager
git add .
git commit -am"Github Mirror Push Back"
#git config pull.rebase true
#git pull
git push

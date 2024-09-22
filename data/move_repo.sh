#! /bin/bash
if [[ "$1" == "deb822" ]]
then
    mv -vf "/tmp/"$2".sources" "/etc/apt/sources.list.d/"$2".sources"
else
    mv -vf "/tmp/"$2".list" "/etc/apt/sources.list.d/"$2".list"
fi
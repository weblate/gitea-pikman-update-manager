#! /bin/bash
if [[ "$1" == "deb822_move" ]]
then
    mv -vf "/tmp/"$2".sources" "/etc/apt/sources.list.d/"$2".sources"
elif  [[ "$1" == "legacy_move" ]]
then
    mv -vf "/tmp/"$2".list" "/etc/apt/sources.list.d/"$2".list"
elif [[ "$1" == "delete_deb822" ]]
then 
    rm -rvf $2
    rm -rvf $3
elif [[ "$1" == "delete_legacy" ]]
then 
    rm -rvf $2
fi
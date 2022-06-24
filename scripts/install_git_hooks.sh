#!bin/bash

rm -rf .git/hooks
ln -s $PWD/.git_hooks .git/hooks

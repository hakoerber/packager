#!/usr/bin/env bash
if [[ $(lsb_release -is) == "Arch" ]] ; then
  sudo pacman -S --needed geckodriver
fi



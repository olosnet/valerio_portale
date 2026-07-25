#!/bin/bash
kitty --session "$(pwd)/launch.kitty-session" & disown && exit

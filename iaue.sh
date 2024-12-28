#!/bin/bash 
 
XDG_DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share} 
 
if [ -d "/opt/system/Tools/PortMaster/" ]; then 
  controlfolder="/opt/system/Tools/PortMaster" 
elif [ -d "/opt/tools/PortMaster/" ]; then 
  controlfolder="/opt/tools/PortMaster" 
elif [ -d "$XDG_DATA_HOME/PortMaster/" ]; then 
  controlfolder="$XDG_DATA_HOME/PortMaster" 
else 
  controlfolder="/roms/ports/PortMaster" 
fi 
 
source $controlfolder/control.txt 
 
#get_controls 
 
$ESUDO chmod 666 /dev/tty1 
$ESUDO chmod 666 /dev/uinput 
 
export SDL_GAMECONTROLLERCONFIG="$sdl_controllerconfig" 
 
export GAMEDIR=$(dirname $(realpath $0))/iaue 
cd $GAMEDIR 
 
#SDL_GAMECONTROLLERCONFIG="19000000010000000100000000010000,Deeplay-keys,a:b3,b:b4,x:b6,y:b5,leftshoulder:b7,rightshoulder:b8,lefttrigger:b13,righttrigger:b14,guide:b11,start:b10,back:b9,dpup:h0.1,dpleft:h0.8,dpright:h0.2,dpdown:h0.4,volumedown:b1,volumeup:b2,leftx:a0,lefty:a1,leftstick:b12,rightx:a2,righty:a3,rightstick:b15,platform:Linux,"  
$GPTOKEYB2 -c $GAMEDIR/iaue.gptk &  
./iaue 
 
kill $(jobs -p)

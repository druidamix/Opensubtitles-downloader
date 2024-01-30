# Opensubtitles Downloader (osd-bin) 
> Cli tool for downloading subtitles from opensubtitles.com using the new API. From 2024, they will turn off the old api.<br>
> Primarily, it downloads subtitles with hash. If not, it downloads the first subtitle found.
## Installation
Archlinux AUR:
```
[yay or paru] -S osd-bin
```
Linux:

```sh
***-It needs rustc version 1.67 or higher-***
git clone https://gitlab.com/marc_ra/opensubtitles-downloader.git
cd opensubtitles-downloader
make
make install
```
## Configure
Register your account on https://www.opensubtitles.com and then create a new api consumer.

Execute osd, so it will generate the conf file on ~/.config/osd.conf

Fill up the osd.conf with your account key, user and pass.

You are done.

## Usage example

usage: osd [-h] [--gui] movie_file

example: osd lord_of_the_rings.mp4

Add --gui as a parameter to choose which subtitle to download. (zenity or kdialog)

Add --custom_title for using a different title 



## Meta

Marc Ratés, druidamix@gmail.com

Distributed under the GPL-3.0 license. See ``LICENSE`` for more information.



# Opensubtitles Downloader (new API) 
> Cli tool for downloading subtitles from opensubtitles.org using the new API.<br>
> Primarely it will download subtitles by hash. If none found, it download the first subtitle found.

## Installation
Archlinux AUR:
```
yay -S osd-bin
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
Register your account on https://www.opensubtitles.com

Then execute osd, so the conf file will be generated on ~/.config/osd.conf

Fill up the osd.conf with your account key, user and pass.

Your are done.

## Usage example

usage: osd [-h] [--gui] movie_file

example: osd lord_of_the_rings.mp4

Add --gui as parameter to choose which subtitle to download. (zenity or kdialog)

## Release History
* 0.1.3 
    * Added verbose  
* 0.1.2
    * Autodetect gtk or qt, so zennity or kdialog will be used
* 0.1.1
    * Added user agent as parameter on osd.conf
* 0.1.0
    * Work in progress and testing

## Meta

Your Name – Marc Ratés, druidamix@gmail.com

Distributed under the GPL 2.0 license. See ``LICENSE`` for more information.



# Opensubtitles Downloader (new API) 
> Download subtitles from opensubtitles.org using the new API for GNU/Linux.<br>
> Primarely it will download subtitles by hash. If none found, it download the first subtitle found.

## Installation

Linux:

```sh
git clone https://gitlab.com/marc_ra/opensubtitles-downloader.git
cd opensubtitles-downloader
make
make install
```
## Configure
Register your account on https://www.opensubtitles.com

Then execute osd, so the conf file will be generated on ~/.config/osd

Fill up the osd.conf with your account key, user and pass.

Your are done.

## Usage example

usage: osd [-h] [--gui] movie_file

example: osd lord_of_the_rings.mp4

Add --gui as parameter to choose wich subtitle to download. (zenity)

## Release History

* 0.1.1
    * Added user agent as parameter on osd.conf
* 0.1.0
    * Work in progress and testing

## Meta

Your Name – Marc Rates, druidamix@gmail.com

Distributed under the GPL 2.0 license. See ``LICENSE`` for more information.



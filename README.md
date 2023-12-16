# Opensubtitles Downloader (new API) 
> Download subtitles from opensubtitles.org using the new API.
> Currently investigating how to create a new account.

## Installation

Linux:

```sh
git clone https://gitlab.com/marc_ra/opensubtitles-new-api-subtitle-downloader.git
cd opensubtitles-new-api-subtitle-downloader
make
make install
```
## Configure
Register your account on https://opensubtitles.stoplight.io/docs/opensubtitles-api/e3750fd63a100-getting-started

Then execute osd, so the conf file will be generated on ~/.config/osd

Fill up the osd.conf with your account 


## Usage example

usage: osd [-h] [--gui] movie_file

example: osd lord_of_the_rings.mp4

## Release History

* 0.0.1
    * Work in progress and testing

## Meta

Your Name – Marc Rates, druidamix@gmail.com

Distributed under the GPL 2.0 license. See ``LICENSE`` for more information.


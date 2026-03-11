# yts-cli
*Minimal TUI app to download YTS movies and opensubtitles subtitles*

## Caveats
- Rust version **1.88**
- It's upon `transmission-rpc` protocol. So It requires to be active in order to use yts-cli. 
- This program has been developed on and for Linux following open source philosophy.

<img src="https://github.com/javiorfo/img/blob/master/bitsmuggler/gativideo.png?raw=true" alt="yts-cli"/>

<img src="https://github.com/javiorfo/img/blob/master/bitsmuggler/gativideo2.png?raw=true" alt="yts-cli"/>

## Installation
- Using Cargo
```bash
cargo install yts-cli
```

- From AUR Arch Linux:
```bash
paru -S yts-cli
```

## Details
- This program is a TUI wrapper of `YTS movies (a.k.a. yify)` and [opensubtitles](https://opensubtitles.org) to search and download movies and subtitles. 
- It uses `transmission-rpc` protocol. Transmission daemon must be configured in order to use **yts-cli**
    - [Transmission configuration](https://github.com/transmission/transmission/blob/main/docs/Editing-Configuration-Files.md)
    - [Transmission How To](https://help.ubuntu.com/community/TransmissionHowTo)
- This program serves itself from crates [yts-lib](https://github.com/javiorfo/yts-lib) and [opensubs](https://github.com/javiorfo/opensubs)
- Multiple movies at the time can be downloaded. You can close **yts-cli** and the downloads still continue.

## Usage
#### Search movie
- Write the name of a movie and press <kbd>Enter</kbd> to search
- Use <kbd>Tab</kbd> to move focus between elements
#### Movies table
- Use <kbd>up</kbd> or <kbd>k</kbd> and <kbd>down</kbd> or <kbd>j</kbd> keys to navigate the table
- Use <kbd>l</kbd> to go to the next page
- Use <kbd>h</kbd> to go to the previous page
- Use <kbd>t</kbd> to open the torrent files popup table
- Use <kbd>s</kbd> to open the subtitles files popup table
- Use <kbd>Tab</kbd> to move focus between elements
#### Popup torrents table
- Use <kbd>up</kbd> or <kbd>k</kbd> and <kbd>down</kbd> or <kbd>j</kbd> keys to navigate the table
- Use <kbd>Enter</kbd> to start the torrent download
- Use <kbd>q</kbd> or <kbd>Esc</kbd> to close the popup
#### Popup subtitles table
- Use <kbd>up</kbd> or <kbd>k</kbd> and <kbd>down</kbd> or <kbd>j</kbd> keys to navigate the table
- Use <kbd>Enter</kbd> to start the subtitle download
- Use <kbd>q</kbd> or <kbd>Esc</kbd> to close the popup
#### Download movies table
- Use <kbd>up</kbd> or <kbd>k</kbd> and <kbd>down</kbd> or <kbd>j</kbd> keys to navigate the table
- Use <kbd>s</kbd> to toggle start/stop a download
- Use <kbd>d</kbd> to delete the download
- Use <kbd>Tab</kbd> to move focus between elements


## Config example
- Some properties could be define in a file stored as `$HOME/.config/yts-cli/config.toml` [default values](https://github.com/javiorfo/yts-cli/blob/master/example/config.toml)
```toml
[yts]
# Default YTS Host if not set
host = "https://en.yts-official.mx"
# Default download dir "$HOME/Downloads" if not set
download_dir = "/home/user/Downloads" 
# Could be "rating" "oldest" "featured" "year" "likes" or "alphabetical" ("rating" is the default)
order = "rating" 

[opensubs]
# Could be a list of languages ("spanish" is the default if not set)
# All the languages are the available in opensubtitles.org 
languages = [ "spanish", "french" ]
# Ordered by "downloads", "uploaded" or "rating"
order = "downloads" 

[transmission]
# Default Transmission RPC host (this is the default if not set)
host = "http://127.0.0.1:9091/transmission/rpc"
# If transmission rpc requires credentials
username = "your_username"
password = "your_password"
```

## Demos and screenshots

https://github.com/user-attachments/assets/a081ee6e-b77d-48d6-8b64-a923a441f5bb

#### Using filters
- **year** filter could be: *from 1920 to 2025*
- **rating** filter could be: *from 1 to 9*
- **order** filter could be: *latest, oldest, rating, alphabetical, featured, year or likes*

<img src="https://github.com/javiorfo/img/blob/master/bitsmuggler/gativideo3.png?raw=true" alt="yts-cli"/>

---

### Donate
- **Bitcoin** [(QR)](https://raw.githubusercontent.com/javiorfo/img/master/crypto/bitcoin.png)  `1GqdJ63RDPE4eJKujHi166FAyigvHu5R7v`
- [Paypal](https://www.paypal.com/donate/?hosted_button_id=FA7SGLSCT2H8G)


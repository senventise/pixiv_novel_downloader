# Pixown
pixown =  **pix**iv novel d**own**load
Download novel from pixiv.

## Features
 - Download novel/series without login.
 - Download all novels from user's bookmarks. (**login needed**)

## Usage
Download a series
```
$ pixown "https://www.pixiv.net/novel/series/xxxxxxxx"
```
Download a single novel
```
$ pixown "https://www.pixiv.net/novel/show.php?id=xxxxxxxx"
```
Download from user's bookmarks
```
# Get cookie from browser, `PHPSESSID` is required.
$ PIXIV_COOKIE=PHPSESSID=xxxx pixown "https://www.pixiv.net/users/xxxxxxxx/bookmarks/novels"
```


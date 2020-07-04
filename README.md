# NewsFlash

The spiritual successor to [FeedReader](https://github.com/jangernert/FeedReader)

A modern feed reader designed for the GNOME desktop. NewsFlash is a program designed to complement an already existing web-based RSS reader account.

It combines all the advantages of web based services like syncing across all your devices with everything you expect
from a modern desktop program: Desktop notifications, fast search and filtering, tagging, handy keyboard shortcuts
and having access to all your articles as long as you like.

![Screenshot](./data/screenshots/Main.png "WIP 2020-04-20")

## Flathub

Official pacakges available on flathub:

<a href='https://flathub.org/apps/details/com.gitlab.newsflash'><img width='240' alt='Download on Flathub' src='https://flathub.org/assets/badges/flathub-badge-en.png'/></a>

## Fedora

Currently available in [COPR](https://copr.fedorainfracloud.org/coprs/atim/newsflash/):

```
sudo dnf copr enable atim/newsflash -y
sudo dnf install newsflash
```

Note: due to missing API secrets it has limited features (e.g. feedly & content scraper).

## Arch Linux

available via Arch User Repository (AUR):

```bash
yay -S newsflash
```

## Looking for service maintainers

I'm looking for people that are actively using a specific service backend of NewsFlash and are willing to maintain it.
The size of the code for each service is quite managable. But keeping an eye on and testing every service can be quite challenging.
So this time around I'm hoping to find at least one person per service that knows the basics of rust and uses the service on a (almost) daily basis.

Services & Maintainers:

- Miniflux: JanGernert (me)
- feedly: still looking
- local RSS: Günther Wagner ([@gunibert](https://gitlab.com/gunibert))
- fever: Felix Bühler ([@Stunkymonkey](https://gitlab.com/Stunkymonkey))
- feedbin: still looking

## Compile
**!!! This is not a supported way of installing the application for normal use. Please use flatpak for that purpose !!!**

Make sure the devel libraries of gtk, webkit2gtk, libhandy, sqlite3, gettext and openssl are installed.
Additionally meson and [rust](https://rustup.rs/) are required.

```
meson --prefix=/usr build
ninja -C build
sudo ninja -C build install
```
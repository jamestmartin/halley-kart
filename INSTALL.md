# Installation Guide
Right now, the only way to install Halley Kart is to install it from source.
A binary distribution will be provided some time in the future.

Currently, these platforms are officially supported:
* Debian GNU/Linux (x86_64)
* Windows (x86_64)

## Debian / Ubuntu
Assuming a completely fresh Debian desktop installation.

Install prerequisite tools and packages:
```
sudo apt install cmake curl gcc g++ git libasound2-dev libjack-jackd2-dev libudev-dev
```

Make sure you have a `python` command using `whereis python`.
If the output of `whereis python` looks like this, with no paths listed:
```
python:
```
Then alias the `python` command to `python3`:
```
sudo update-alternatives --install /usr/bin/python python /user/bin/python3 2
```

Install the latest version of Rust stable using [https://rustup.rs](rustup)
and follow the provided instructions.

Download Halley Kart's source code:
```
git clone https://github.com/jamestmartin/halley-kart.git && cd halley-kart
```

Build the game:
```
cargo build --release`
```

Finally, you can run the game:
```
cargo run --release
```

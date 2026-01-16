noisy-silence
=============
[![Crates.io](https://img.shields.io/crates/v/noisy-silence?logo=rust&style=flat-square&logoColor=white "Crates.io")](https://crates.io/crates/noisy-silence)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/kijewski/noisy-silence/ci.yml?branch=main&logo=github&style=flat-square&logoColor=white "GitHub Workflow Status")](https://github.com/kijewski/noisy-silence/actions/workflows/ci.yml)

My USB headset has an irritating, quiet but very perceptible, high-pitched
noise when no audio stream is playing. This program emits an even quieter,
imperceptible, brownian noise to keep the USB audio line busy. This suppresses
the high-pitched noise.

Maybe your audio set up has the same problem? If so, then maybe this program
could be helpful to you! :)

**Alternatively:**
You can run the program with arguments like ``noisy-silence white 10`` to
play *white* noise with an amplitude of *10%*. Instead of *white*, you can
choose out of *white*, *gaussian*, *triangular*, *pink*, *blue*, *violet*,
*brownian*, and *velvet*. The amplitude must be in a range of 0.01 to 100,
but you probably want to keep it well below 25.

You can find an explanation for the noise types [here](
https://docs.rs/rodio/0.21.1/rodio/source/noise/index.html "Noise sources for audio synthesis and testing.").

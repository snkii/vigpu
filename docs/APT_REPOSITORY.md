# APT repository operations

ViGPU publishes signed Debian packages through the GitHub Pages site at `https://seon.uk/vigpu`.

## Repository signing key

The public key is committed at `apt/vigpu-archive-keyring.asc` and is copied to the Pages site by the release workflow. The matching private key is stored only as the GitHub Actions repository secret `APT_GPG_PRIVATE_KEY`. Never commit or print the private key.

The current public-key fingerprint is `3EA8 7FCF 697F 0E00 B3D3 9415 288D C568 CF86 B6B8`. Verify it with `gpg --show-keys --with-fingerprint apt/vigpu-archive-keyring.asc` before trusting the key.

To rotate the key, generate a replacement, publish its public half before switching the workflow secret, then retain the old public key during a transition release so existing installations can update safely.

## Publishing

Pushing a version tag such as `v0.1.0` triggers `.github/workflows/release.yml`. The workflow builds a `.deb`, attaches it to a GitHub Release, creates `Packages` and signed `InRelease` metadata, and deploys the APT repository to GitHub Pages. Release assets receive a GitHub artifact attestation.

The repository currently targets `amd64` on Debian 12+ and Ubuntu 22.04+. The package requires an NVIDIA driver with NVML available at runtime.

## Client setup

```sh
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://seon.uk/vigpu/vigpu-archive-keyring.asc \
  | sudo gpg --dearmor -o /etc/apt/keyrings/vigpu.gpg
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/vigpu.gpg] https://seon.uk/vigpu stable main" \
  | sudo tee /etc/apt/sources.list.d/vigpu.list
sudo apt update
sudo apt install vigpu
```

The APT repository serves Debian and Ubuntu. Other Linux families need packages for their native package managers.

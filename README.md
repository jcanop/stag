<p align="center">
    <img src="images/logo.png" alt="Stag Logo" />
</p>

This command-line utility lets users embed and extract data from images using steganography. [Steganography](https://en.wikipedia.org/wiki/Steganography) is the practice of hiding a secret message, file, or image inside another ordinary, non-secret file or physical object to avoid detection; in this case, image files.

Even if the embedded data is hidden, it is easy to extract if you know the algorithm, so this is not a way to securely protect your data. For a little more security, the application supports password-based encryption, so you will need a password to embed and extract your data from the image.

This is a fun little project I built over a weekend, so there are some limitations; for example, while multiple image file formats are supported as an input, the output is always PNG.

## Compiling the source code.

You will need the Rust compiler and the cargo utility to build this project.

```bash
cargo build --release
```

## Usage

### Display help
```bash
stag -h
```

### Embed data into an image
```basg
stag embed [OPTIONS] <INPUT\_IMAGE> <OUTPUT\_IMAGE>

Arguments:
  <INPUT\_IMAGE>   Input image
  <OUTPUT\_IMAGE>  Output image

Options:
  -i, --input <FILE>           Input filename
  -e, --encrypt                Encrypt input data
  -p, --password <PASSPHRASE>  Encryption password
```

Examples:
```bash
# The application will prompt the user for an input text and embed it into the input.jpg image file and write the result into output.png
stag embed input.jpg output.png

# Embed the message.txt file into the input.jpg image file and write the result into output.png
stag embed --input message.txt input.jpg output.png

# Embed and encrypt the message.txt file into the input.jpg image file and write the result into output.png
stag embed --encrypt --input message.txt input.jpg output.png
```

Even though these examples embed only text, you can embed any file format.

### Extract data from an image
```bash
stag extract [OPTIONS] <INPU\T_IMAGE>

Arguments:
  <INPUT\_IMAGE>  Input image

Options:
  -o, --output <FILE>          Output filename
  -p, --password <PASSPHRASE>  Decrypt password
  -t, --text                   Display the output in the console
```

Examples:
```bash
# Extract the hidden message from input.png and display it in the console.
stag extract --text input.png

# Extract the hidden message from input.png and write it to output.txt.
stag extract --output output.txt input.png
```


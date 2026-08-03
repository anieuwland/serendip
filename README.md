Serendip
--------

Read Fluke is2 thermal imagery. Used by Blackbody ([Linux](https://flathub.org/en/apps/eu.nimmerfort.blackbody),
[Windows](https://bitbucket.org/nimmerwoner/blackbody/downloads/)) to read and render Fluke files.

## Features

* Export to kelvin
* Access standard Fluke markers
* Access the visual light images

## Supported cameras

* Ti400, Ti401p
* TiS75+

_**Request for help**_: Please share any is2 files of cameras not listed above so 
this library can be tested against them and be improved.

## TODO

* [x] Expose markers
* [x] Support is2 zip format
* [x] Expose visual light imagery
* [ ] Support is2 blob format
* [ ] Validate against more camera models (share samples!)
* [ ] Expose metadata (camera info, exif if any, ...)
* [ ] CLI to export temperatures to file

## Example

### Read a Fluke thermogram

```rust
use serendip::SerendipThermogram::{self, Zip};

let file_path = "thermograms/fluke_ti400_1.is2"
match SerendipThermogram::new_from_path(Path::new(&file_path)) {
    Ok(Zip(thermogram)) => {
        println!("Successfully decoded thermogram from {file_path}");
        println!("{:?}", thermogram.kelvin()); // Use however you want
        ExitCode::SUCCESS
    }
    Err(e) => {
        eprintln!("Failed to decode {file_path}: {e}");
        ExitCode::FAILURE
    }
}
```

### Access visual lights images

```rust
use serendip::SerendipThermogram;

let file_path = "thermograms/fluke_ti400_3.is2"
let Ok(thermogram) = SerendipThermogram::new_from_path(Path::new(&file_path));
let visuals = thermogram.visuals(); // Raw encoded bytes
let decoded = thermogram.visual_decoded(); // Get image decoded
```

## Prior art

* **[fluke-thermal-reader](https://github.com/LoriGH25/Fluke-Thermal-Reader_Python)**. A Python library ([on PyPI](https://pypi.org/project/fluke-thermal-reader/)) for reading `.is2` files, with partial `.is3` (video) support. 
* **[IS2 tool](https://github.com/joshuahamsa/IS2-Tool)**. A PyQt-based desktop application for reviewing and renaming `.is2` files.
* **[goconvertis2](https://github.com/weisskopfjens/goconvertis2)**. A Go command-line tool and package extracting visible and infrared images from `.is2` files. Handles both format generations: the older monolithic binary format and the newer ZIP-based container.
* **read-fluke-is2-images-from-is3**. A community script for reading `.is2` files. [BaurA's GitLab fork](https://gitlab.com/BaurA/read-fluke-is2-images-from-is3) extends it to read `.is2` frames extracted from `.is3` videos via Fluke Connect.
* **[EEVblog forum threads](https://www.eevblog.com/forum/testgear/fluke-vt02-is2-file-format-specification/)**. Reverse-engineering discussion (VT02, Ti-series) going back to 2014.
* **[JackieHanLab/ThermoFace](https://github.com/JackieHanLab/ThermoFace)**. Contains pre-processing code for Fluke Ti401 Pro imagery.

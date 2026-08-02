Serendip
--------

Read Fluke is2 thermal imagery. Used by Blackbody ([Linux](https://flathub.org/en/apps/eu.nimmerfort.blackbody),
[Windows](https://bitbucket.org/nimmerwoner/blackbody/downloads/)) to read and reader Fluke files.

## Features

* Export to kelvin

## Supported cameras

*Request for help*: Please share any is2 files of cameras not listed below so 
this library can be tested against them and be improved.

* Ti400, Ti401p

## TODO

* [ ] Expose visual light imagery
* [ ] Expose metadata (camera info, exif if any, ...)
* [ ] Expose markers
* [ ] CLI to export temperatures to file

## Prior art

* **[fluke-thermal-reader](https://github.com/LoriGH25/Fluke-Thermal-Reader_Python)**. A Python library ([on PyPI](https://pypi.org/project/fluke-thermal-reader/)) for reading `.is2` files, with partial `.is3` (video) support. The most complete reverse-engineering effort: handles protobuf-encoded calibration blobs (`CalibrationData.gpbenc`, `CalTempDataRex.gpbenc`) and per-model quirks (Ti480P, Ti300, TiS75+, PTi120). Also documents the unsolved proprietary `V_FLUKE/HUFF` thermal video codec. MIT licensed.
* **[IS2 tool](https://github.com/joshuahamsa/IS2-Tool)**. A PyQt-based desktop application for reviewing and renaming `.is2` files.
* **[goconvertis2](https://github.com/weisskopfjens/goconvertis2)**. A Go command-line tool and package extracting visible and infrared images from `.is2` files. Handles both format generations: the older monolithic binary format and the newer ZIP-based container. Temperature values reportedly somewhat inaccurate. MIT licensed.
* **read-fluke-is2-images-from-is3**. An older, oft-forked community script for reading `.is2` files. [BaurA's GitLab fork](https://gitlab.com/BaurA/read-fluke-is2-images-from-is3) extends it to read `.is2` frames extracted from `.is3` videos via Fluke Connect.
* **[EEVblog forum threads](https://www.eevblog.com/forum/testgear/fluke-vt02-is2-file-format-specification/)**. Reverse-engineering discussion (VT02, Ti-series) going back to 2014.
* **[JackieHanLab/ThermoFace](https://github.com/JackieHanLab/ThermoFace)**. Contains pre-processing code for Fluke Ti401 Pro imagery.

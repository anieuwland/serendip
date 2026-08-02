Serendip
--------

Serendip is a rust library to read Fluke thermal imagery.

## Format documentation

Fluke publishes no documentation for the `.is2` format. An unofficial specification, compiled from the prior art below, is maintained in [`docs/is2-format.md`](docs/is2-format.md).

## Prior art

* **[fluke-thermal-reader](https://github.com/LoriGH25/Fluke-Thermal-Reader_Python)**. A Python library ([on PyPI](https://pypi.org/project/fluke-thermal-reader/)) for reading `.is2` files, with partial `.is3` (video) support. The most complete reverse-engineering effort: handles protobuf-encoded calibration blobs (`CalibrationData.gpbenc`, `CalTempDataRex.gpbenc`) and per-model quirks (Ti480P, Ti300, TiS75+, PTi120). Also documents the unsolved proprietary `V_FLUKE/HUFF` thermal video codec. MIT licensed.
* **[IS2 tool](https://github.com/joshuahamsa/IS2-Tool)**. A PyQt-based desktop application for reviewing and renaming `.is2` files.
* **[goconvertis2](https://github.com/weisskopfjens/goconvertis2)**. A Go command-line tool and package extracting visible and infrared images from `.is2` files. Handles both format generations: the older monolithic binary format and the newer ZIP-based container. Temperature values reportedly somewhat inaccurate. MIT licensed.
* **read-fluke-is2-images-from-is3**. An older, oft-forked community script for reading `.is2` files. [BaurA's GitLab fork](https://gitlab.com/BaurA/read-fluke-is2-images-from-is3) extends it to read `.is2` frames extracted from `.is3` videos via Fluke Connect.
* **[EEVblog forum threads](https://www.eevblog.com/forum/testgear/fluke-vt02-is2-file-format-specification/)**. Reverse-engineering discussion (VT02, Ti-series) going back to 2014.

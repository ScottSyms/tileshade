# tileshade

A simple tile server for visualizing large datasets of points, built with Python, FastAPI, and Datashader.

This web service reads geospatial data from a CSV file, and serves map tiles as PNG images. The color of the tiles represents the density of the data points.

## Features

*   **Fast Tile Generation:** Uses Datashader for high-performance tile generation.
*   **CSV File Support:** Reads data from a CSV file.
*   **Web Mercator Projection:** Converts longitude/latitude to Web Mercator coordinates for tile generation.
*   **Simple Frontend:** A basic frontend is provided in the `www` directory to visualize the tiles.

## Requirements

*   Python 3.7+
*   `datashader`
*   `pyarrow`
*   `fastapi`
*   `pandas`
*   `colorcet`
*   `uvicorn`

## Installation

1.  Clone the repository:
    ```bash
    git clone https://github.com/ScottSyms/tileshade.git
    cd tileshade
    ```

2.  Install the dependencies:
    ```bash
    pip install -r requirements.txt
    ```
    *(Note: A `requirements.txt` file does not exist yet. You will need to create one with the dependencies listed above.)*

## Usage

1.  Get the data by running the `getdata.sh` script:
    ```bash
    ./getdata.sh
    ```
    This will download the data and place it in `data/stored.csv`.

2.  Run the server:
    ```bash
    ./run.sh
    ```
    Or directly with uvicorn:
    ```bash
    uvicorn main:app --host 0.0.0.0 --port 8000
    ```

3.  Open your browser and navigate to `http://localhost:8000` to see the tiles.

## Endpoints

*   `GET /`: Serves the frontend.
*   `GET /tiles/{zoom}/{x}/{y}.png`: Generates and serves a map tile.

## License

This project is licensed under the terms of the LICENSE file.
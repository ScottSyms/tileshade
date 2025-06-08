import io
import math
import os
import pathlib
import sys

import datashader as ds
from pyarrow import csv
import fastapi
import pandas as pd
from pandas.core.frame import DataFrame
from colorcet import bmw, coolwarm, fire
from datashader import transfer_functions as tf
from datashader.utils import lnglat_to_meters
from fastapi import FastAPI, Response
from fastapi.responses import FileResponse, HTMLResponse, StreamingResponse
from fastapi.staticfiles import StaticFiles

#from PIL import Image, ImageDraw
from starlette.responses import FileResponse


def tile2mercator(xtile, ytile, zoom):
    # takes the zoom and tile path and passes back the EPSG:3857
    # coordinates of the top left of the tile.
    # From Openstreetmap
    n = 2.0 ** zoom
    lon_deg = xtile / n * 360.0 - 180.0
    lat_rad = math.atan(math.sinh(math.pi * (1 - 2 * ytile / n)))
    lat_deg = math.degrees(lat_rad)

    # Convert the results of the degree calulation above and convert
    # to meters for web map presentation
    mercator = lnglat_to_meters(lon_deg, lat_deg)
    return mercator


def generateatile(zoom, x, y):
    # The function takes the zoom and tile path from the web request,
    # and determines the top left and bottom right coordinates of the tile.
    # This information is used to query against the dataframe.
    xleft, yleft = tile2mercator(int(x), int(y), int(zoom))
    xright, yright = tile2mercator(int(x)+1, int(y)+1, int(zoom))
    condition = '(X >= {xleft}) & (X <= {xright}) & (Y <= {yleft}) & (Y >= {yright})'.format(
        xleft=xleft, yleft=yleft, xright=xright, yright=yright)
    frame = td.query(condition)
    # The dataframe query gets passed to Datashder to construct the graphic.
    # First the graphic is created, then the dataframe is passed to the Datashader aggregator.
    csv = ds.Canvas(plot_width=256, plot_height=256, x_range=(min(xleft, xright), max(
        xleft, xright)), y_range=(min(yleft, yright), max(yleft, yright)))
    agg = csv.points(frame, 'X', 'Y')
    # The image is created from the aggregate object, a color map and aggregation function.
    # Then the object is assighed to a bytestream and returned
    # Use a 3-colour gradient that shifts from blue to yellow and then red.
    # Histogram equalisation helps accentuate changes at low zoom levels.
    img = tf.shade(agg, cmap=["#0000FF", "#FFFF00", "#FF0000"], how='eq_hist')

    img_io = img.to_bytesio('PNG')
    img_io.seek(0)
    bytes = img_io.read()
    return bytes




# Start the web server
app = FastAPI()

@app.on_event("startup")
async def startup_event():
    global td, indexhtml

    # Load file
    indexhtml = open('./www/index.html', 'r').read()

    if pathlib.Path("data/stored.csv").exists():
        print("Loading data from file...")
        #td = pd.read_csv('data/stored.csv', usecols=['X', 'Y'])
        pp = csv.read_csv('data/stored.csv')
        td = pp.to_pandas()

        #td=td.set_index(['X', 'Y'])
    else:
        print("There's no data/stored.csv file!  Execute ./getdata.sh first!")
        sys.exit(1)


@app.get("/", response_class=HTMLResponse)
async def root():
    return indexhtml

@app.get("/tiles/{zoom}/{x}/{y}.png")
async def gentile(zoom, x, y):
    results = generateatile(zoom, x, y)
    return Response(content=results, media_type="image/png")

app.mount("/lib", StaticFiles(directory="./www/lib"), name="lib")

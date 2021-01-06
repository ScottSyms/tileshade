#!/bin/bash
#ogr2ogr -f CSV output.csv -lco GEOMETRY=AS_XY -s_srs EPSG:4269 -t_srs EPSG:3857 -progress -oo X_POSSIBLE_NAMES=LON -oo Y_POSSIBLE_NAMES=LAT test.csv
# One one wonder- using ogr2ogr, pull a data file from Marine Cadastre, reproject it and output a
# three column extract in data/stored.csv

spin()
{
  spinner="/|\\-/|\\-"
  while :
  do
    for i in `seq 0 7`
    do
      echo -n "${spinner:$i:1}"
      echo -en "\010"
      sleep .5
    done
  done
}

#
# flags
# -f CSV - source file is in comma separated values format
# -lco GEOMETRY=AS_XY - point data is encoded as X (Longitude) and Y (Latitude) pairs.
# -s_srs EPSG:4269 - source projection
# -t_srs EPSG:3857 - target projection
# -oo X_POSSIBLE_NAMES=LON - designate "LON" as the longitude column
# -oo Y_POSSIBLE_NAMES=LON - designate "LAT" as the latitude column
# data/stored.csv - where to store the output 
# /vsizip - unzip the results of the retrieve
# /vsicurl - retrieve the data from a remote web site
# /https://coast.noaa.gov/htdata/CMSP/AISDataHandler/2020/AIS_2020_01_01.zip - the location of the file
echo "Retrieving data from Marine Cadastre.. this can take 15 minutes..."

# Enable spinner
spin &
SPIN_PID=$!
trap "kill -9 $SPIN_PID" `seq 0 15`

clear

echo "Grabbing data from Marine Cadastre..."
curl https://coast.noaa.gov/htdata/CMSP/AISDataHandler/2020/AIS_2020_06_28.zip -o data/raw.zip

echo "Decompressing file..."
unzip data/raw.zip -d data

echo "Reprojecting the data ... can take  10-15 minutes...."
time ogr2ogr -f CSV -lco GEOMETRY=AS_XY -select BaseDateTime -s_srs EPSG:4269 -t_srs EPSG:3857 -oo X_POSSIBLE_NAMES=LON -oo Y_POSSIBLE_NAMES=LAT data/AISstored.csv data/AIS_2020_06_28.csv

echo "Trimming extra data from conversion ..."
cat data/AISstored.csv |  awk -F ',' '{print $1","$2}' > data/stored.csv

echo "Cleaning up...."
rm data/AIS*.csv 2>/dev/null
rm data/*.zip 2>/dev/null
rm data/raw.csv 2>/dev/null

# Terminate the spinner stored2.csv
kill -9 $SPIN_PID

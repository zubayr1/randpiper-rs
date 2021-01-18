#!/usr/bin/python3

# A script that takes raw output from our experiment script and produces a
# CSV file that can be used by others for other things.
#
# Usage
# Raw input files as input or stdin
# Output csv files
import argparse
import sys
from numpy import percentile
# from csv import writer
from json import dump

def filter_data(s: str):
    splits = s.split(",")
    key = splits[0]
    x = splits[1]
    y = splits[2]
    if y.count("ms") == 1:
        y = y.split(" ")[0]
        y = float(y)
    elif y.count("us") == 1:
        y = y.split(" ")[0]
        y = float(y)/1000.0
    else:
        y = y.split(" ")[0]
        y = float(y)*1000
    return key, x, y

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Program to convert raw data of a bench experiment run into a CSV file for plotting")
    parser.add_argument('input', nargs='?', type=argparse.FileType('r'),
                    default=sys.stdin)
    parser.add_argument('output', nargs='?', type=argparse.FileType('w'),
                    default=sys.stdout)
    args = parser.parse_args()
    data = {}
    while True:
        line = args.input.readline()
        if line == "":
            break
        key,x,y = filter_data(line)
        # print(key,x,y)
        if key not in data:
            data[key] = {}
            data[key]["x"] = [x]
            data[key]["y"] = [y]
        else:
            data[key]["x"].append(x)
            data[key]["y"].append(y)
    # print(data)
    # outfile = writer(args.output)
    # for d in data:
        # outfile.writerow()
    dump(data, args.output)
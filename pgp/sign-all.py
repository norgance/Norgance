import argparse
import glob
import os

import sign

def main():
  # create parser
  parser = argparse.ArgumentParser()
  
  # add arguments to the parser
  parser.add_argument("directory")

  # parse the arguments
  args = parser.parse_args()

  for input_file in glob.iglob(os.path.join(args.directory, "**/*.html"), recursive=True):
    with open(input_file) as input:
      print(input_file)
      sign.sign(input)

if __name__ == "__main__":
    main()
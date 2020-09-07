import argparse
import os
import tempfile

def main():
  # create parser
  parser = argparse.ArgumentParser()
  
  # add arguments to the parser
  parser.add_argument("--input", metavar="i",type=open, required=True)
  parser.add_argument("--output", metavar="o", type=argparse.FileType("w", encoding="UTF-8"), required=False)
 
  # parse the arguments
  args = parser.parse_args()
  sign(args.input, args.output)

def sign(input, output = None):
  content = input.read()

  override_mode = output == None

  with tempfile.TemporaryDirectory() as workdir:
    prepared_path = os.path.join(workdir, 'prepared')
    signed_path = os.path.join(workdir, 'signed')

    with open(prepared_path, mode="w") as prepared:
      prepared.write("https://norgance.com/pgp/ -->\n")
      prepared.write(content)
      prepared.write("<!--")

    os.system("gpg --digest-algo SHA256 --output "+signed_path+" --clearsign "+ prepared_path)

    with open(signed_path) as signed:
      signed_content = signed.read()
  
  if override_mode:
    output = open(input.name, mode="w")

  output.write("<!--\n")
  output.write(signed_content[:-1])
  output.write("\n-->")

  if override_mode:
    output.close()

if __name__ == "__main__":
  main()
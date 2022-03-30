import tomli

with open("autonomous_mail_client.toml") as f:
    config = tomli.load(f)
    # while line:=f.readline():

print(config)

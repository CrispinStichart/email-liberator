import sys
import json

# x = """
# {
#     "4": 5,
#     "6": 7
# }"""
# j = json.loads(x)

j = json.loads(sys.argv[1])
print(j["body"], end="")

# print(json.dumps(j, indent=2))

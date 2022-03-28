# hotedit

## Install

```
pip install hotedit
```

## Use

```
import requests

from hotedit import hotedit

URL = "https://pastebin.com/raw/Df9NAmYc"

response = requests.get(URL)
edited = hotedit(response.text)

print("Your edited text:")
for line in edited.splitlines():
    print(f"> {line}")
```

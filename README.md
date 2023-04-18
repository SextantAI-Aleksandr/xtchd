# xtchd

***Cutting through disinformation, one cryptographically verified item at a time.*** 

Consider these phrases that have entered our common lexicon in the past few years: "fake news", "misinformation", "stealth edit", "shadow banning", "AI-generated". The prevalence of these phrases demonstrates that distrust of news outlets, institutions, and expertise have reached societally corrosive levels. 

What can we do to help ensure the "believability" of a new item? xtchd seeks to provide a platform with enhanced believability via two approaches:



### Cryptographically verifying all content via a hash chain

This basically means that no content in xtchd can **ever** be edited or deleted, and that furthermore this immutability can be publicly verified. Each time an item is added,  a SHA-256 hash digest is calculated for the JSON object plus the date it was uploaded plus the prior sha256:

**Example Input for sha256**

```
{"title":"Fabio saw a turtle", "text":"He said it was green."} item_id=123 uploaded_at=2023.04.02_12:13:59 prior_sha256=da27f72459e28584777ab03fef4297aa4484f518da4edb8b643d90cfc6d8c8ec
```

**Example Output from sha256:**

```449833ff65faff7c4c5f683b30002ee926bde48ae06be152fa19608314edf333```

You can copy and paste the example above into this [free online sha256-hash-calculator](https://xorbin.com/tools/sha256-hash-calculator) and verify that it gives the expected result. While you are there, try changing anything in the input then re-calculate the sha256 hash: notice how it won't match! 



### The more diverse the audience, the harder it is to lie

One lie might be told in isolation. But the broader the audience it is exposed to, the higher the number of other narratives it has to be consistent with, the harder it is to conceal any inconsistencies. For this reason, xtchd places heavy emphasis on the connectivity of sources referencing other sources. This also helps the user reason about the logic in a given article: The sources are all shown.





## How to verify sha256 integrity in various languages

#### Online

See [https://xorbin.com/tools/sha256-hash-calculator](https://xorbin.com/tools/sha256-hash-calculator) for a useful tool to sanity-check your hash values.



#### JavaScript

The function below is the uncommented version of that given in `js/hashIntegrity.js` It is based on the [SubtleCrypto](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/digest) library.

```javascript
async function sha256(message) {
    const msgBuffer = new TextEncoder().encode(message);                    
    const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));              
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    return hashHex;
}

// pass a message and await the promise
var hashPromise = sha256("hello world!");
// prints 7509e5bda0c762d2bac7f90d758b5b2263fa01ccbc542ab5e3df163be08e6ca9
hashPromise.then((resp) => console.log(resp)); 

```



#### Rust

```rust
use sha2::{Sha256, Digest}; // Digest brings the ::new() method into scope

fn sha256(input: &str) -> String {                                                         
    let mut hasher = Sha256::new();                                                       
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result) // lowercase hexadecimal encoding
}

// returns "7509e5bda0c762d2bac7f90d758b5b2263fa01ccbc542ab5e3df163be08e6ca9"
sha256("hello world!") 
```



#### Python

```python 
import hashlib 

def hash256_hex(input: str):
    hash = hashlib.sha256(input.encode('utf-8'))
    return hash.hexdigest()

# returns '7509e5bda0c762d2bac7f90d758b5b2263fa01ccbc542ab5e3df163be08e6ca9'
hash256_hex('hello world!') 
```





#### Postgres

```sql
SELECT encode(sha256('hello world!'::bytea), 'hex');
                              encode                              
------------------------------------------------------------------
 7509e5bda0c762d2bac7f90d758b5b2263fa01ccbc542ab5e3df163be08e6ca9
(1 row)
```


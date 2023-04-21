# xtchd



***'Gaslighting' was Merriam-Webster's word of the year for 2022*** [link](https://www.npr.org/2022/11/28/1139384432/its-no-trick-merriam-webster-says-gaslighting-is-the-word-of-the-year). They give the top definition for *gaslighting* as *"psychological manipulation of a person usually over an extended period of time that causes the victim to question the validity of their own thoughts, perception of reality, or memories and typically leads to confusion, loss of confidence and self-esteem, uncertainty of one's emotional or mental stability, and a dependency on the perpetrator"*.

The selection of that word should perhaps not be surprising. Consider these phrases that have entered our common lexicon in the past few years: "fake news", "misinformation", "stealth edit", "shadow banning", "AI-generated". The prevalence of these phrases demonstrates that distrust of news outlets, institutions, and expertise in general have reached societally corrosive levels. A common theme is that people are growing suspicious that they have been played.

What can we do to help ensure the "believability" of content, of news and of reports about what someone said earlier? We hope that, perhaps, **xtchd** (pronounced "etched") can be part of that solution. **Xtchd** focuses on:



### Cryptographic verification of all content via hash chains

Each time an author, an article, or a reference is added in **xtchd**, one (or more) "links" are appended to a "hash chain". This makes it possible to mathematically **prove** that *nothing has been edited, and nothing has been deleted*. In fact, some clever constraints in the data storage schema makes it *impossible* to delete or edit data. 



### Make many references and show them all 

One lie might be told in isolation. But the broader the audience it is exposed to, the higher the number of other narratives it has to be consistent with, the harder it is to conceal any inconsistencies.  In addition to making extensive use of references, **xtchd** goes further by  

1) **Harmonizing** references, so when two articles reference the same thing, you know about it at a glance.
2) **Cryptographically verifying** the references in each article, like everything else. 

This strong emphasis on references also helps the user reason about the logic in a given article: The sources are all shown.



### What is a hash and how do I calculate it?

A hash is a fixed-length output mathematically calculated from variable-length input that will be *almost* unique based on that input. What this means is that, if you change even one letter of the input, the output looks completely different. **By verifying the output hash is as expected,** you can therefore verify that the content of the article has not been altered. 

A hash chain extends this concept by *including the output hash from the prior item in the input for the next*. This means that you can verify not just that an article has not been modified, but that *all articles have not been modified and further that none have been deleted*. 

**Xtchd** uses the sha256 hash which is robust and widely utilized. If you want to verify the hash integrity something yourself, here is how to do so in several languages:

#### Online

See [https://xorbin.com/tools/sha256-hash-calculator](https://xorbin.com/tools/sha256-hash-calculator) for a useful tool to sanity-check your hash values.

#### JavaScript

The function below is based on the [SubtleCrypto](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/digest) library:

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

Most of **xtchr** is written in *Rust*. Here is the function used in ```src/integrity.rs```:

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

The canonical database for **xtchd** is Postgres,  albeit with some rather clever (we think) constraints that make it impossible to edit or delete rows (see ```schema.sql```). Postgres can also calculate sha256 values:

```sql
SELECT encode(sha256('hello world!'::bytea), 'hex');
                              encode                              
------------------------------------------------------------------
 7509e5bda0c762d2bac7f90d758b5b2263fa01ccbc542ab5e3df163be08e6ca9
(1 row)
```


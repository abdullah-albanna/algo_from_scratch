## base64 simple implementation in rust

### give it a try

**setup**

```bash
git clone https://github.com/abdullah-albanna/base64 && cd base64
```

---

**from stdio**

```bash
echo "hey" | cargo r 
```
prints out ```aGV5Cg==```


**from a file**
```bash
cargo r -- Cargo.toml
```

prints out 

```
W3BhY2thZ2VdCm5hbWUgPSAiYmFzZTY0Igp2ZXJzaW9uID0gIjAuMS4wIgplZGl0aW9uID0gIjIwMjEiCgpbZGVwZW5kZW5jaWVzXQpjbGFwID0geyB2ZXJzaW9uID0gIjQiLCBmZWF0dXJlID0gImRlcml2ZSIsIGZlYXR1cmVzID0gWyJkZXJpdmUiXSB9CmFueWhvdyA9ICIxIgoK
```

----

**you can chain them**

```bash
cat Cargo.toml | cargo r | cargo r -- -d
```

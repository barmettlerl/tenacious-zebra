# Tenacious zebra
An efficient and persitent key-value store. 

## Description of the algorithm

### Patricia Trie (Prefix tree, Radix tree)
source: [merkel-patricia-trie](https://medium.com/codechain/modified-merkle-patricia-trie-how-ethereum-saves-a-state-e6d7555078dd)

Trie uses a **key** as a path so the nodes that share the same prefix can also share the same path. This structure is fastest at finding common prefixes, simple to implement, and requires small memory.

<p align="center">
  <img src="./doc/patricia_trie.png" width="300" title="patricia tree">
</p>

### Merkel Tree (Hash tree)
source: [merkel-patricia-trie](https://medium.com/codechain/modified-merkle-patricia-trie-how-ethereum-saves-a-state-e6d7555078dd)

Merkle tree is a tree of hashes. Leaf nodes store data. Parent nodes contain their children’s hash as well as the hashed value of the sum of their children’s hashes. 

Finding out whether two different nodes have the same data or not can be efficiently done with the Merkle tree. You first have to compare the Top Hash value of the two nodes. If they are the same, then the two nodes have same data. For example, if you look at the picture above, when there are four nodes (L1, L2, L3, L4), you only need to check whether they have the same Top Hash or not. If the Top Hash is different and you want to know which data is different, you should compare Hash 0 with Hash1 and check which branch is different. By doing so, you will eventually find out which data is different.

<p align="center">
  <img src="./doc/merkel_tree.png" width="500" title="merkel tree">
</p>

### Merkle Patricia Trie
It's a combination of the Patricia Tree's structure and the cryptographic verification and efficiency of the Merkle Tree.

- **Storing Data:** When data is added to the tree, it's split up and each piece of data gets its path based on its content. This ensures that every piece of data has a unique path.
- **Verification:** Due to the Merkle properties, you can prove that a particular piece of data is in the tree without seeing the entire tree. You only need a path from your data up to the top (the Merkle proofs). If the hashes along the way match up, you can be certain that your piece of data is indeed in the tree.
- **Changing Data:** When data in the tree is updated or changed, only the path from the changed data to the root will need to be recalculated. This ensures efficiency when updating the tree.
- **Hashing:** Each node in the tree is hashed, starting from the leaf nodes and moving up to the root. This creates a cascading effect; a small change in a leaf node will produce a completely different root hash.
- **Root Hash:** The root hash represents the entire state. In Ethereum, this allows for quick and cryptographic verification of any part of the data.

The primary advantages of the Merkle Patricia Tree in Ethereum are:

- Efficiency: It minimizes storage by avoiding storing redundant chunks of data.

- Cryptography: It allows for quick and secure verification of large sets of data.

- Decentralization: Its cryptographic properties are perfect for decentralized systems like blockchains where verification and trust are paramount.

### Parallel execution
<p align="center">
  <img src="./doc/tree_overview.jpg" width="600" title="parallel execution of merkel-patricia tree">
</p>


### Properties of the data structure / algorithm
- Concurrent processing of operations on different keys with minimal thread synchronization.
- Cheap cloning (O(1)).
- Efficient sending to `Databases` containing similar maps (high % of key-value pairs in common)
- Quick validation of the correctness of a tree 
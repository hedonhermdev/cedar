# Cedar

Cedar provides an easy to setup, in-memory vector database that you can embed in your Rust application.

```rust
    // 1. initialize db
    let db = DuckDB::new(Default::default())?;
    db.init()?;

    // 2. initialize embedding function (could be OpenAI, Chrome, etc)
    let embedding_fn = SentenceTransformerEmbeddings::new();
    // Or use OpenAI embeddings:
    let embedding_fn = OpenAIEmbeddingFunction::new(
        "<api_key>".to_string(),
    );

    // 3. initialize client
    let mut client = LocalClient::init(db, embedding_fn)?;

    // 4. create a collection
    let mut collection = client.create_collection("collection1")?;

    // 5. push documents to the store
    let docs = &[
        Document {
            text: "this is about macbooks".to_string(),
            metadata: json!({ "source": "laptops" }),
            id: Uuid::new_v4(),
        },
        Document {
            text: "lychees are better than mangoes".to_string(),
            metadata: json!({ "source": "facts" }),
            id: Uuid::new_v4(),
        },
    ];
    collection.add_documents(docs)?;

    // 6. query the vector store for matching documents
    let k = 1;
    let res = collection.query_documents(&["which one is the better fruit?"], k, json!({ "source": "facts" }))?;
```

# Installation

To use cedar in your project, start with adding it to your `Cargo.toml`. (Standalone cedar server coming soon!)

```toml
[dependencies]
cedar = "0.1.0"
```

`cedar` uses the `tch-rs` bindings for PyTorch. To set up the bindings, follow these steps:

1. Download `libtorch` from https://pytorch.org/get-started/locally/. This package requires `v2.0.0`: if this version is no longer available on the "get started" page, the file should be accessible by modifying the target link, for example `https://download.pytorch.org/libtorch/cu118/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcu118.zip` for a Linux version with CUDA11. **NOTE:** When using `rust-bert` as dependency from [crates.io](https://crates.io), please check the required `LIBTORCH` on the published package [readme](https://crates.io/crates/rust-bert) as it may differ from the version documented here (applying to the current repository version).

2. Extract the library to a location of your choice


3. Set the following environment variables

### Linux:
```bash
export LIBTORCH=/path/to/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
```

### macOS + Homebrew
```bash
brew install pytorch jq
export LIBTORCH=$(brew --cellar pytorch)/$(brew info --json pytorch | jq -r '.[0].installed[0].version')
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
```

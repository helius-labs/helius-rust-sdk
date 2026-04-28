//! Microbenchmark comparing `serde_json` vs `simd-json` on Helius-shaped responses.
//!
//! Run with: `cargo bench --bench json_parsing`
//!
//! The fixtures here are synthetic but mirror the shape and rough size of real
//! responses from `searchAssets` / `getAssetsByOwner` (DAS) and
//! `getProgramAccountsV2` — the payloads Jacopo flagged as the latency hot spot.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use helius::types::{ApiResponse, AssetList, GetProgramAccountsV2Response};

fn build_das_payload(n_assets: usize) -> Vec<u8> {
    let asset = serde_json::json!({
        "interface": "V1_NFT",
        "id": "HcaSe8RfGASYdPws2NFu7y84C1mWAhsdnhDDSWAQGfmh",
        "content": {
            "$schema": "https://schema.metaplex.com/nft1.0.json",
            "json_uri": "https://www.hi-hi.vip/json/3000jup.json",
            "files": [{
                "uri": "https://img.hi-hi.vip/json/img/3000jup.png",
                "mime": "image/png",
                "cdn_uri": "https://cdn.helius-rpc.com/cdn-cgi/image//https://img.hi-hi.vip/json/img/3000jup.png"
            }],
            "metadata": {
                "attributes": [
                    {"trait_type": "Website", "value": "https://3000jup.com"},
                    {"trait_type": "Verified", "value": "true"},
                    {"trait_type": "Amount",   "value": "3000 JUP"}
                ],
                "description": "Synthetic asset for benchmarking",
                "name": "3000 JUP",
                "symbol": "JUP",
                "token_standard": "NonFungible"
            },
            "links": {
                "image": "https://img.hi-hi.vip/json/img/3000jup.png",
                "external_url": "https://3000jup.com"
            }
        },
        "authorities": [{
            "address": "HnT5KVAywGgQDhmh6Usk4bxRg4RwKxCK4jmECyaDth5R",
            "scopes": ["full"]
        }],
        "compression": {
            "eligible": false,
            "compressed": false,
            "data_hash": "",
            "creator_hash": "",
            "asset_hash": "",
            "tree": "",
            "seq": 0,
            "leaf_id": 0
        },
        "grouping": [{
            "group_key": "collection",
            "group_value": "J2ZfLdQsaZ3GCmBpKLPiJ6F5fmFwgGgK1aVxvR8wRbMs"
        }],
        "royalty": {
            "royalty_model": "creators",
            "target": null,
            "percent": 0.05,
            "basis_points": 500,
            "primary_sale_happened": true,
            "locked": false
        },
        "creators": [{
            "address": "5Mj6CmeGnGNNh9wMPtCcppsR1f4yNh3p7vEtkEGxJ8Hv",
            "share": 100,
            "verified": true
        }],
        "ownership": {
            "frozen": false,
            "delegated": false,
            "delegate": null,
            "ownership_model": "single",
            "owner": "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY"
        },
        "supply": {
            "print_max_supply": 0,
            "print_current_supply": 0,
            "edition_nonce": 252
        },
        "mutable": true,
        "burnt": false
    });

    let items: Vec<_> = (0..n_assets).map(|_| asset.clone()).collect();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "bench",
        "result": {
            "total": n_assets,
            "limit": n_assets,
            "page": 1,
            "items": items
        }
    });

    serde_json::to_vec(&payload).expect("serialize fixture")
}

fn build_gpa_v2_payload(n_accounts: usize) -> Vec<u8> {
    let account = serde_json::json!({
        "pubkey": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
        "account": {
            "lamports": 2039280u64,
            "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "data": ["c29tZWJhc2U2NGRhdGFmb3JhdG9rZW5hY2NvdW50YmVuY2htYXJrcGF5bG9hZA==", "base64"],
            "executable": false,
            "rentEpoch": 18446744073709551615u64,
            "space": 165
        }
    });

    let accounts: Vec<_> = (0..n_accounts).map(|_| account.clone()).collect();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "bench",
        "result": {
            "context": {"slot": 343001234, "apiVersion": "2.1.0"},
            "value": {
                "accounts": accounts,
                "paginationKey": "next-cursor-token",
                "totalResults": n_accounts
            }
        }
    });

    serde_json::to_vec(&payload).expect("serialize fixture")
}

fn bench_das(c: &mut Criterion) {
    let mut group = c.benchmark_group("das_search_assets");
    for size in [10usize, 100, 1000] {
        let bytes = build_das_payload(size);
        group.throughput(Throughput::Bytes(bytes.len() as u64));

        group.bench_with_input(BenchmarkId::new("serde_json", size), &bytes, |b, bytes| {
            b.iter(|| {
                let resp: ApiResponse<AssetList> =
                    serde_json::from_slice(black_box(bytes.as_slice())).expect("serde_json parse");
                black_box(resp);
            });
        });

        group.bench_with_input(BenchmarkId::new("simd_json", size), &bytes, |b, bytes| {
            b.iter_batched(
                || bytes.clone(),
                |mut buf| {
                    let resp: ApiResponse<AssetList> =
                        simd_json::serde::from_slice(black_box(&mut buf)).expect("simd_json parse");
                    black_box(resp);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_gpa_v2(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_program_accounts_v2");
    for size in [10usize, 100, 1000] {
        let bytes = build_gpa_v2_payload(size);
        group.throughput(Throughput::Bytes(bytes.len() as u64));

        group.bench_with_input(BenchmarkId::new("serde_json", size), &bytes, |b, bytes| {
            b.iter(|| {
                let resp: ApiResponse<GetProgramAccountsV2Response> =
                    serde_json::from_slice(black_box(bytes.as_slice())).expect("serde_json parse");
                black_box(resp);
            });
        });

        group.bench_with_input(BenchmarkId::new("simd_json", size), &bytes, |b, bytes| {
            b.iter_batched(
                || bytes.clone(),
                |mut buf| {
                    let resp: ApiResponse<GetProgramAccountsV2Response> =
                        simd_json::serde::from_slice(black_box(&mut buf)).expect("simd_json parse");
                    black_box(resp);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_das, bench_gpa_v2);
criterion_main!(benches);

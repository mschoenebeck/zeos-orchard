window.BENCHMARK_DATA = {
  "lastUpdate": 1657444334657,
  "repoUrl": "https://github.com/mschoenebeck/zeos-orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "matthias.schoenebeck@gmail.com",
            "name": "mschoenebeck",
            "username": "mschoenebeck"
          },
          "committer": {
            "email": "matthias.schoenebeck@gmail.com",
            "name": "mschoenebeck",
            "username": "mschoenebeck"
          },
          "distinct": true,
          "id": "1bbea3e373c1ba58dea748aa6cf808f15f773942",
          "message": "extended circuit design: added C_D1 for burnft2 and g_d_c, pk_d_c to be able to redirect 'change' from transferft and burnft to other wallet",
          "timestamp": "2022-07-10T03:59:53-05:00",
          "tree_id": "27d6871ef6205b5a0ae79841a45637033f88cb9e",
          "url": "https://github.com/mschoenebeck/zeos-orchard/commit/1bbea3e373c1ba58dea748aa6cf808f15f773942"
        },
        "date": 1657444332739,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4837670090,
            "range": "± 43679158",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4836931624,
            "range": "± 14658408",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 6926507677,
            "range": "± 13858391",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 8992682856,
            "range": "± 45088794",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}
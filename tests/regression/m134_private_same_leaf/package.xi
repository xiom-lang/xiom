package {
  name: "m134-private-same-leaf"
  version: "0.1.0"
  description: "Relay lock: PRIVATE same-leaf types across project modules must not share one layout"
  modules: [
    "m134.main",
    "m134.alpha",
    "m134.beta",
  ]
}

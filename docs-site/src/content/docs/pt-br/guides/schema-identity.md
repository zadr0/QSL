---
title: Identidade do esquema
description: Os hashes estrito e leve do esquema, o que entra neles e como compará-los em um handshake.
---

O QSL calcula dois hashes para cada tipo durante a análise do esquema e os guarda como constantes no código gerado. A codificação e a decodificação nunca os calculam, e os frames não os contêm.

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // também <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
Hash | Cobre
---|---
Estrito | Nomes do tipo e dos campos, ordem dos campos, tipos, intervalos, opcionalidade e opções de formato.
Leve | O mesmo, sem os nomes do tipo e dos campos.
`&A { u32 id; }` e `&B { u32 sequence; }` têm o mesmo hash leve e hashes estritos diferentes. Trocar de lugar dois campos de tipos diferentes muda os dois.

`SCHEMA_HASHES` é um valor `qsl::SchemaHashes`:
Campo | Significado
---|---
`version` | Versão do formato de entrada do hash, atualmente 1.
`algorithm` | `Xxh3_64` ou `Xxh3_128`.
`strict`, `light` | Os hashes. Com XXH3-64, os 64 bits superiores são zero.
`complete` | `false` se o tipo usa um tipo [`QslCustom`](/guides/custom-types/#qslcustom). Nesse caso, o hash cobre só o caminho Rust e o tamanho do tipo personalizado, não o codec dele.
Compare `version`, `algorithm` e `complete` junto com o hash. `strict_hex()` e `light_hex()` formatam um hash como 16 ou 32 dígitos hexadecimais, conforme o algoritmo.

## Algoritmo

XXH3-64 por padrão. Para 128 bits, configure em um [arquivo DEQ](/language/deq/):

```deq
!config USE_HASH = xxHash128;
```

A configuração vale para o build inteiro. Os dois algoritmos não são criptográficos: eles detectam divergências, não adulteração.

## O que muda o hash

- A sequência de campos depois da avaliação das constantes e da expansão dos templates Rust.
- Tipos, intervalos de valores, limites de strings e arrays, e quais campos são opcionais.
- Opções de formato binário: `compact`, `alignment` junto com `compact`, `optionals=bitmap`, `predict_seq`, `predict_seq` com `opt_table`, `opt_table=legacy`, `state_bits`.
- Codecs de string: `decompress=true` e `decompress=true`.
- `qps` e o quantum dele. `[qps]`, `[qps=Q16]` e `[quant=Q16]` são equivalentes.
- No formato padrão de campos opcionais, as tags dos campos opcionais e a largura da tag.

## O que não muda

`safety`, `lifetime`, `inline` e suas variantes, `default`, `opt_table` sozinho, `alignment` sem `compact`, `aggressive`, `transmutate`, `cache` e `loyal_cache`, `QSL_RT` e o número de slots. Eles mudam o código Rust gerado ou o tratamento da memória, não os bytes.

`String`, `&str`, `StackString` e `lifetime` produzem os mesmos bytes para os mesmos limites, por isso têm o mesmo hash.

## Como usar

Os hashes descrevem o formato de forma conservadora. Eles não provam que dois esquemas diferentes não conseguem decodificar os mesmos bytes. Para garantir que os dois lados concordam, troque os hashes no seu próprio handshake antes de enviar dados. O QSL não oferece handshake.

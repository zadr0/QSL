---
title: Erros
description: As variantes de QslError e quando cada uma é retornada.
---

Os métodos gerados retornam `qsl::QslResult<T>`, um alias de `Result<T, QslError>`. `QslError` tem 11 variantes e implementa `std::error::Error`. A saída de `Display` é `QSL error: <Variant>`, por exemplo `QSL error: InvalidUtf8`.
Variante | Retornada quando
---|---
`UnexpectedEof` | A entrada termina antes do frame.
`BufferTooSmall` | Um buffer que você passou é pequeno demais para a saída.
`LengthOverflow` | A aritmética de comprimento causaria overflow.
`InvalidWidth` | Uma largura ou um quantum não é suportado.
`InvalidValue` | Um valor, um comprimento ou um campo de metadados está fora do domínio permitido.
`InvalidUtf8` | Os bytes da string não são UTF-8 válido.
`InvalidSchema` | O esquema é inválido, ou a operação não se aplica a ele. Também é o resultado padrão dos validadores de `QslCustom`.
`InvalidPadding` | Bits ou bytes de padding não são zero.
`PoolExhausted` | Todos os slots RT estão ocupados. Só vem do `try_acquire`, que não espera.
`SizeLimitExceeded` | O frame ou os dados decodificados passam de `MAX_DATA_SIZE`.
`AllocationFailed` | Uma alocação falível falhou.
Operações com arquivos e sockets também podem retornar `std::io::Error`.

Uma validação bem-sucedida significa que os bytes se encaixam no formato do esquema. Ela não diz quem os enviou nem com qual esquema foram escritos.

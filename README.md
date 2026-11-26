# Carbonado

[![Versão](https://img.shields.io/github/v/release/RafaelWergilyds/carbonado?label=vers%C3%A3o)](https://github.com/RafaelWergilyds/carbonado/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/RafaelWergilyds/carbonado/total)](https://github.com/RafaelWergilyds/carbonado/releases)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6)

Compressor de vídeo desktop leve e rápido. Você escolhe a **qualidade mínima** e o Carbonado comprime o máximo possível sem passar desse ponto.

- H.264 ou H.265, com GPU NVIDIA (NVENC) e fallback automático para CPU
- Tamanho estimado em tempo real enquanto você ajusta a qualidade
- Redução de resolução (1080p/720p/480p), sem nunca aumentar o vídeo
- Vídeos HDR convertidos para SDR automaticamente
- Interface em português e inglês
- FFmpeg embutido: não é preciso instalar nada

Feito com Rust + Tauri v2 e Svelte 5 + TypeScript.

## Download

**[Baixar para Windows](https://github.com/RafaelWergilyds/carbonado/releases/latest)**: baixe o `Carbonado_x.x.x_x64-setup.exe` e execute.

- Requer Windows 10/11 64-bit.
- O Windows pode mostrar o aviso do SmartScreen porque o instalador não é assinado. Clique em **Mais informações → Executar assim mesmo**.
- A aceleração por GPU exige uma placa NVIDIA com driver atualizado. Sem ela, o app usa a CPU.

## Desenvolvimento

Pré-requisitos:
- [Rust](https://rustup.rs) e Visual Studio Build Tools (carga *Desenvolvimento para desktop com C++*)
- Node 20+ e pnpm
- FFmpeg em `src-tauri/binaries/` (não versionado). Baixe a build **n8.1 GPL** do [BtbN](https://github.com/BtbN/FFmpeg-Builds/releases) e renomeie:
  - `ffmpeg.exe` → `ffmpeg-x86_64-pc-windows-msvc.exe`
  - `ffprobe.exe` → `ffprobe-x86_64-pc-windows-msvc.exe`

```sh
pnpm install
pnpm tauri dev                  # rodar o app
pnpm tauri build                # gerar instaladores (src-tauri/target/release/bundle)
pnpm check                      # type-check do frontend
cd src-tauri && cargo test      # testes do backend
```

Para publicar uma versão nova, atualize `version` em `package.json` e `src-tauri/tauri.conf.json`, rode `pnpm tauri build` e anexe o instalador a uma [release](https://github.com/RafaelWergilyds/carbonado/releases/new) com a tag `vX.Y.Z`.

## Estrutura

```
src/                     Interface (Svelte)
  lib/api.ts             chamadas ao backend
  lib/i18n.svelte.ts     traduções pt/en
  lib/stores/            estado da tela
  lib/components/        componentes
src-tauri/src/           Backend (Rust)
  commands/              comandos chamados pela interface
  domain/                lógica pura: qualidade → CRF/CQ, args do ffmpeg, estimativa
  media/                 execução do ffmpeg/ffprobe e detecção de GPU
```

## Qualidade

| Encoder | Qualidade 100 → 0 |
|---|---|
| libx264 | CRF 18 → 32 |
| libx265 | CRF 20 → 34 |
| h264_nvenc | CQ 19 → 33 |
| hevc_nvenc | CQ 21 → 35 |

Presets: Baixa = 35, Equilibrada = 60, Alta = 85.

## Licença do FFmpeg

O Carbonado inclui o [FFmpeg](https://ffmpeg.org) (build GPL do [BtbN](https://github.com/BtbN/FFmpeg-Builds)), distribuído sob a [GPL v3](https://www.gnu.org/licenses/gpl-3.0.html). O código-fonte do FFmpeg está em <https://github.com/FFmpeg/FFmpeg>. Veja `src-tauri/binaries/FFMPEG-LICENSE.txt`.

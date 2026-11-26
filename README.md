# Carbonado

Compressor de vídeo desktop leve e rápido. Você escolhe a **qualidade mínima** e o Carbonado comprime o máximo possível sem passar desse ponto.

- H.264 ou H.265, com GPU NVIDIA (NVENC) e fallback automático para CPU
- Tamanho estimado em tempo real enquanto você ajusta a qualidade
- Redução de resolução (1080p/720p/480p), sem nunca aumentar o vídeo
- Vídeos HDR convertidos para SDR automaticamente
- Interface em português e inglês
- FFmpeg embutido: não é preciso instalar nada

Feito com Rust + Tauri v2 e Svelte 5 + TypeScript.

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

A build GPL do FFmpeg traz obrigações da GPL ao distribuir o app. Veja `src-tauri/binaries/FFMPEG-LICENSE.txt`.

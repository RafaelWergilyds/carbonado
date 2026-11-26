import type { AppError } from './types';

export type Locale = 'pt' | 'en';

const pt = {
  'titlebar.checkingGpu': 'Verificando GPU…',
  'titlebar.gpu': 'GPU NVENC',
  'titlebar.cpu': 'CPU',
  'titlebar.gpuUnavailable': 'Aceleração por GPU indisponível; usando CPU',
  'titlebar.minimize': 'Minimizar',
  'titlebar.maximize': 'Maximizar',
  'titlebar.restore': 'Restaurar',
  'titlebar.close': 'Fechar',
  'titlebar.language': 'Idioma',

  'drop.reading': 'Lendo vídeo…',
  'drop.wait': 'Só um instante.',
  'drop.release': 'Solte para carregar',
  'drop.drag': 'Arraste um vídeo para cá',
  'drop.formats': 'MP4, MOV, MKV, AVI, WebM e outros',
  'drop.choose': 'Escolher vídeo',
  'drop.filter': 'Vídeos',

  'video.hdr': 'HDR → SDR',
  'video.hdrHint': 'O vídeo será convertido para SDR, compatível com qualquer tela',

  'settings.quality': 'Qualidade mínima',
  'settings.preset.low': 'Baixa',
  'settings.preset.balanced': 'Equilibrada',
  'settings.preset.high': 'Alta',
  'settings.preset.custom': 'Personalizada',
  'settings.estimate': '≈ {size} estimado (−{pct})',
  'settings.estimateBigger': '≈ {size} estimado · maior que o original',
  'settings.estimateFailed': 'Não foi possível estimar o tamanho',
  'settings.estimating': 'Estimando tamanho…',
  'settings.codec': 'Codec',
  'settings.codec.h264Hint': 'Abre em qualquer lugar',
  'settings.codec.h265Hint': 'Arquivos ~40% menores',
  'settings.resolution': 'Resolução',
  'settings.resolution.original': 'Original',
  'settings.resolution.noUpscale': 'O vídeo nunca é ampliado',
  'settings.gpu': 'Aceleração por GPU',
  'settings.gpu.checking': 'Verificando placa de vídeo…',
  'settings.gpu.available': 'NVENC: bem mais rápido, com arquivos um pouco maiores que na CPU.',
  'settings.gpu.unavailable': 'Indisponível para este codec neste computador. A CPU será usada.',

  'output.title': 'Salvar em',
  'output.none': 'Nenhum local escolhido',
  'output.change': 'Alterar',
  'output.dialogTitle': 'Salvar vídeo comprimido',
  'output.filter': 'Vídeo MP4',

  'progress.title': 'Comprimindo',
  'progress.speed': 'Velocidade {speed}',
  'progress.remaining': 'Faltam {time}',
  'progress.calculating': 'Calculando tempo restante…',
  'progress.savingAs': 'Salvando como',
  'progress.gpuFallback': 'A GPU falhou ao codificar; continuando com a CPU (mais lento).',

  'result.done': 'Concluído',
  'result.skipped': 'Nada a comprimir',
  'result.skippedText': 'O vídeo já está mais compacto do que a qualidade escolhida permite. Diminua a qualidade ou a resolução.',
  'result.reveal': 'Mostrar na pasta',

  'actions.changeVideo': 'Trocar vídeo',
  'actions.compress': 'Comprimir',
  'actions.keepWorking': 'Você pode continuar usando o computador.',
  'actions.cancel': 'Cancelar',
  'actions.adjust': 'Ajustar e refazer',
  'actions.another': 'Comprimir outro vídeo',
  'actions.chooseAnother': 'Escolher outro vídeo',
  'actions.retry': 'Tentar novamente',

  'close.title': 'Fechar o Carbonado?',
  'close.message': 'A compressão em andamento será cancelada e o progresso descartado.',
  'close.confirm': 'Fechar',
  'close.cancel': 'Continuar comprimindo',

  'error.title': 'Erro',
  'error.details': 'Detalhes técnicos',
  'error.unknown': 'Algo deu errado',
  'error.ffmpegNotFound': 'Não foi possível iniciar o FFmpeg',
  'error.probeFailed': 'Não foi possível ler o vídeo',
  'error.encodeFailed': 'A compressão falhou',
  'error.io': 'Erro ao gravar o arquivo',
  'error.alreadyRunning': 'Já existe uma compressão em andamento',
  'error.fileNotFound': 'Arquivo não encontrado',
  'error.notAFile': 'O caminho escolhido não é um arquivo',
  'error.noVideoStream': 'O arquivo não contém vídeo',
  'error.unknownDuration': 'Não foi possível descobrir a duração do vídeo',
  'error.outputIsInput': 'O destino não pode ser o próprio vídeo original',
  'error.outputDirMissing': 'A pasta de destino não existe',
  'error.qualityOutOfRange': 'A qualidade deve estar entre 0 e 100',
  'error.unsupportedFormat': 'Formato de arquivo não suportado',
} as const;

export type MessageKey = keyof typeof pt;

const en: Record<MessageKey, string> = {
  'titlebar.checkingGpu': 'Checking GPU…',
  'titlebar.gpu': 'GPU NVENC',
  'titlebar.cpu': 'CPU',
  'titlebar.gpuUnavailable': 'GPU acceleration unavailable; using CPU',
  'titlebar.minimize': 'Minimize',
  'titlebar.maximize': 'Maximize',
  'titlebar.restore': 'Restore',
  'titlebar.close': 'Close',
  'titlebar.language': 'Language',

  'drop.reading': 'Reading video…',
  'drop.wait': 'Just a moment.',
  'drop.release': 'Drop to load',
  'drop.drag': 'Drag a video here',
  'drop.formats': 'MP4, MOV, MKV, AVI, WebM and more',
  'drop.choose': 'Choose video',
  'drop.filter': 'Videos',

  'video.hdr': 'HDR → SDR',
  'video.hdrHint': 'The video will be converted to SDR so it looks right on any screen',

  'settings.quality': 'Minimum quality',
  'settings.preset.low': 'Low',
  'settings.preset.balanced': 'Balanced',
  'settings.preset.high': 'High',
  'settings.preset.custom': 'Custom',
  'settings.estimate': '≈ {size} estimated (−{pct})',
  'settings.estimateBigger': '≈ {size} estimated · larger than the original',
  'settings.estimateFailed': 'Could not estimate the size',
  'settings.estimating': 'Estimating size…',
  'settings.codec': 'Codec',
  'settings.codec.h264Hint': 'Plays everywhere',
  'settings.codec.h265Hint': 'Files ~40% smaller',
  'settings.resolution': 'Resolution',
  'settings.resolution.original': 'Original',
  'settings.resolution.noUpscale': 'Videos are never upscaled',
  'settings.gpu': 'GPU acceleration',
  'settings.gpu.checking': 'Checking graphics card…',
  'settings.gpu.available': 'NVENC: much faster, with files slightly larger than on the CPU.',
  'settings.gpu.unavailable': 'Not available for this codec on this computer. The CPU will be used.',

  'output.title': 'Save to',
  'output.none': 'No location chosen',
  'output.change': 'Change',
  'output.dialogTitle': 'Save compressed video',
  'output.filter': 'MP4 video',

  'progress.title': 'Compressing',
  'progress.speed': 'Speed {speed}',
  'progress.remaining': '{time} left',
  'progress.calculating': 'Calculating time left…',
  'progress.savingAs': 'Saving as',
  'progress.gpuFallback': 'The GPU failed to encode; continuing on the CPU (slower).',

  'result.done': 'Done',
  'result.skipped': 'Nothing to compress',
  'result.skippedText': 'This video is already more compact than the chosen quality allows. Lower the quality or the resolution.',
  'result.reveal': 'Show in folder',

  'actions.changeVideo': 'Change video',
  'actions.compress': 'Compress',
  'actions.keepWorking': 'You can keep using your computer.',
  'actions.cancel': 'Cancel',
  'actions.adjust': 'Adjust and redo',
  'actions.another': 'Compress another video',
  'actions.chooseAnother': 'Choose another video',
  'actions.retry': 'Try again',

  'close.title': 'Close Carbonado?',
  'close.message': 'The compression in progress will be cancelled and its progress discarded.',
  'close.confirm': 'Close',
  'close.cancel': 'Keep compressing',

  'error.title': 'Error',
  'error.details': 'Technical details',
  'error.unknown': 'Something went wrong',
  'error.ffmpegNotFound': 'FFmpeg could not be started',
  'error.probeFailed': 'Could not read the video',
  'error.encodeFailed': 'Compression failed',
  'error.io': 'Could not write the file',
  'error.alreadyRunning': 'A compression is already running',
  'error.fileNotFound': 'File not found',
  'error.notAFile': 'The chosen path is not a file',
  'error.noVideoStream': 'The file has no video',
  'error.unknownDuration': 'Could not determine the video length',
  'error.outputIsInput': 'The destination cannot be the original video',
  'error.outputDirMissing': 'The destination folder does not exist',
  'error.qualityOutOfRange': 'Quality must be between 0 and 100',
  'error.unsupportedFormat': 'Unsupported file format',
};

const dictionaries: Record<Locale, Record<MessageKey, string>> = { pt, en };

const STORAGE_KEY = 'carbonado.locale';

function initialLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === 'pt' || saved === 'en') return saved;
  } catch {
    return 'pt';
  }
  return navigator.language.toLowerCase().startsWith('pt') ? 'pt' : 'en';
}

class I18n {
  locale = $state<Locale>(initialLocale());

  constructor() {
    this.applyDocumentLang();
  }

  set(locale: Locale) {
    this.locale = locale;
    this.applyDocumentLang();
    try {
      localStorage.setItem(STORAGE_KEY, locale);
    } catch {
      return;
    }
  }

  get intl(): string {
    return this.locale === 'pt' ? 'pt-BR' : 'en-US';
  }

  private applyDocumentLang() {
    if (typeof document !== 'undefined') document.documentElement.lang = this.intl;
  }
}

export const i18n = new I18n();

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  const template = dictionaries[i18n.locale][key];
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, name: string) => String(params[name] ?? `{${name}}`));
}

export function errorMessage(error: AppError | null): string {
  if (!error) return t('error.unknown');
  for (const candidate of [error.code, error.kind]) {
    const key = `error.${candidate}` as MessageKey;
    if (candidate && key in pt) return t(key);
  }
  return t('error.unknown');
}

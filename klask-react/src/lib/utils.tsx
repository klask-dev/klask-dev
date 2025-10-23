import { formatDistanceToNow as fnsFormatDistanceToNow, format, parseISO } from 'date-fns';
import {
  DocumentIcon,
  DocumentTextIcon,
  CodeBracketIcon,
  PhotoIcon,
  FilmIcon,
  MusicalNoteIcon,
  ArchiveBoxIcon,
  CogIcon,
} from '@heroicons/react/24/outline';

// File size formatting
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
};

// Date formatting
export const formatDistanceToNow = (date: string | Date): string => {
  try {
    return fnsFormatDistanceToNow(new Date(date), { addSuffix: true });
  } catch {
    return 'Unknown time';
  }
};

// Format datetime with both date and time using browser's locale
// Backend sends UTC, browser automatically converts to local timezone
export const formatDateTime = (
  dateString: string | null | undefined,
  options?: Intl.DateTimeFormatOptions
): string => {
  if (!dateString) return '-';

  try {
    const date = parseISO(dateString);
    const defaultOptions: Intl.DateTimeFormatOptions = {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      ...options,
    };
    return date.toLocaleString(undefined, defaultOptions);
  } catch {
    return 'Invalid date';
  }
};

// Format date only (no time) using browser's locale
export const formatDate = (
  dateString: string | null | undefined,
  options?: Intl.DateTimeFormatOptions
): string => {
  if (!dateString) return '-';

  try {
    const date = parseISO(dateString);
    const defaultOptions: Intl.DateTimeFormatOptions = {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      ...options,
    };
    return date.toLocaleDateString(undefined, defaultOptions);
  } catch {
    return 'Invalid date';
  }
};

// Get programming language from file extension
export const getLanguageFromExtension = (extension: string): string => {
  const ext = extension.toLowerCase().replace('.', '');
  
  const languageMap: Record<string, string> = {
    // Web
    js: 'javascript',
    jsx: 'jsx',
    ts: 'typescript',
    tsx: 'tsx',
    html: 'html',
    htm: 'html',
    css: 'css',
    scss: 'scss',
    sass: 'scss',
    less: 'less',
    
    // Backend
    py: 'python',
    java: 'java',
    c: 'c',
    cpp: 'cpp',
    'c++': 'cpp',
    cc: 'cpp',
    h: 'c',
    hpp: 'cpp',
    cs: 'csharp',
    php: 'php',
    rb: 'ruby',
    go: 'go',
    rs: 'rust',
    swift: 'swift',
    kt: 'kotlin',
    scala: 'scala',
    dart: 'dart',
    
    // Markup/Data
    xml: 'xml',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    toml: 'toml',
    ini: 'ini',
    conf: 'ini',
    
    // Shell/Scripts
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    fish: 'bash',
    ps1: 'powershell',
    
    // Database
    sql: 'sql',
    
    // Documentation
    md: 'markdown',
    mdx: 'mdx',
    txt: 'text',
    
    // Config
    dockerfile: 'dockerfile',
    gitignore: 'gitignore',
    
    // Frameworks
    vue: 'vue',
    svelte: 'svelte',
  };
  
  return languageMap[ext] || 'text';
};

// Get file icon based on extension
export const getFileIcon = (extension: string, className: string = 'h-5 w-5') => {
  const ext = extension.toLowerCase().replace('.', '');
  
  // Programming/code files
  if (['js', 'jsx', 'ts', 'tsx', 'py', 'java', 'c', 'cpp', 'h', 'hpp', 'cs', 'php', 'rb', 'go', 'rs', 'swift', 'kt', 'scala', 'dart'].includes(ext)) {
    return <CodeBracketIcon className={`${className} text-blue-500`} />;
  }
  
  // Web files
  if (['html', 'htm', 'css', 'scss', 'sass', 'less'].includes(ext)) {
    return <DocumentTextIcon className={`${className} text-orange-500`} />;
  }
  
  // Images
  if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext)) {
    return <PhotoIcon className={`${className} text-green-500`} />;
  }
  
  // Videos
  if (['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv', 'flv', 'wmv'].includes(ext)) {
    return <FilmIcon className={`${className} text-purple-500`} />;
  }
  
  // Audio
  if (['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a'].includes(ext)) {
    return <MusicalNoteIcon className={`${className} text-pink-500`} />;
  }
  
  // Archives
  if (['zip', 'tar', 'gz', 'rar', '7z', 'bz2', 'xz'].includes(ext)) {
    return <ArchiveBoxIcon className={`${className} text-yellow-500`} />;
  }
  
  // Config files
  if (['json', 'yaml', 'yml', 'toml', 'ini', 'conf', 'env'].includes(ext)) {
    return <CogIcon className={`${className} text-slate-500`} />;
  }
  
  // Text/Documentation
  if (['md', 'txt', 'readme'].includes(ext)) {
    return <DocumentTextIcon className={`${className} text-slate-600`} />;
  }
  
  // Default
  return <DocumentIcon className={`${className} text-slate-400`} />;
};

// Generate breadcrumb from file path
export const generateBreadcrumb = (path: string): Array<{ name: string; path: string }> => {
  if (!path || path === '/') {
    return [{ name: 'Root', path: '/' }];
  }
  
  const parts = path.split('/').filter(Boolean);
  const breadcrumbs = [{ name: 'Root', path: '/' }];
  
  let currentPath = '';
  for (const part of parts) {
    currentPath += `/${part}`;
    breadcrumbs.push({
      name: part,
      path: currentPath,
    });
  }
  
  return breadcrumbs;
};

// Determine if file is binary
export const isBinaryFile = (extension: string): boolean => {
  const ext = extension.toLowerCase().replace('.', '');
  
  const binaryExtensions = [
    // Images
    'jpg', 'jpeg', 'png', 'gif', 'bmp', 'ico', 'tiff', 'webp',
    // Videos
    'mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm', 'ogg',
    // Audio
    'mp3', 'wav', 'flac', 'aac', 'm4a', 'wma',
    // Archives
    'zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz',
    // Executables
    'exe', 'dll', 'so', 'dylib', 'app',
    // Office documents
    'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'pdf',
    // Other binary
    'bin', 'dat', 'db', 'sqlite', 'sqlite3',
  ];
  
  return binaryExtensions.includes(ext);
};

// Get file type category
export const getFileCategory = (extension: string): string => {
  const ext = extension.toLowerCase().replace('.', '');
  
  if (['js', 'jsx', 'ts', 'tsx', 'py', 'java', 'c', 'cpp', 'h', 'hpp', 'cs', 'php', 'rb', 'go', 'rs', 'swift', 'kt', 'scala', 'dart', 'html', 'css', 'scss', 'sass'].includes(ext)) {
    return 'code';
  }
  
  if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext)) {
    return 'image';
  }
  
  if (['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv', 'flv', 'wmv'].includes(ext)) {
    return 'video';
  }
  
  if (['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a'].includes(ext)) {
    return 'audio';
  }
  
  if (['md', 'txt', 'readme'].includes(ext)) {
    return 'document';
  }
  
  if (['json', 'yaml', 'yml', 'toml', 'ini', 'conf', 'env'].includes(ext)) {
    return 'config';
  }
  
  if (['zip', 'tar', 'gz', 'rar', '7z', 'bz2', 'xz'].includes(ext)) {
    return 'archive';
  }
  
  return 'other';
};

// Truncate text with ellipsis
export const truncateText = (text: string, maxLength: number): string => {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + '...';
};

// Highlight search term in text
export const highlightSearchTerm = (text: string, searchTerm: string): string => {
  if (!searchTerm) return text;
  
  const regex = new RegExp(`(${searchTerm})`, 'gi');
  return text.replace(regex, '<mark>$1</mark>');
};

// Class name utility (similar to clsx)
export const cn = (...classes: Array<string | undefined | null | false>): string => {
  return classes.filter(Boolean).join(' ');
};
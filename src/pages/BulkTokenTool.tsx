import React, { useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Upload,
  CheckCircle2,
  XCircle,
  Copy,
  Download,
  Eye,
  EyeOff,
  Loader2,
  FileJson,
  Trash2,
  AlertTriangle,
} from 'lucide-react';
import { request as invoke } from '../utils/request';
import { useTranslation } from 'react-i18next';

// Types matching backend BulkTokenEntry/BulkTokenResult/BulkTokenResponse
interface BulkTokenEntry {
  username: string;
  refresh_token: string;
}

interface BulkTokenResult {
  username: string;
  email: string | null;
  api_key: string | null;
  account_id: string | null;
  status: string;
  error: string | null;
}

interface BulkTokenResponse {
  total: number;
  success_count: number;
  failed_count: number;
  results: BulkTokenResult[];
}

// Input format example
const EXAMPLE_INPUT = `[
  { "username": "user1", "refresh_token": "1//0abc..." },
  { "username": "user2", "refresh_token": "1//0def..." }
]`;

export default function BulkTokenTool() {
  const { t } = useTranslation();
  const [inputText, setInputText] = useState('');
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [response, setResponse] = useState<BulkTokenResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [revealedKeys, setRevealedKeys] = useState<Set<number>>(new Set());
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  // Parse input (JSON or line format: username|refresh_token)
  const parseInput = useCallback((text: string): BulkTokenEntry[] => {
    const trimmed = text.trim();
    if (!trimmed) throw new Error('Input is empty');

    // Try JSON parse first
    try {
      const parsed = JSON.parse(trimmed);
      if (Array.isArray(parsed)) {
        return parsed.map((item: any, idx: number) => {
          if (!item.username || !item.refresh_token) {
            throw new Error(`Entry ${idx + 1} missing username or refresh_token`);
          }
          return { username: item.username, refresh_token: item.refresh_token };
        });
      }
    } catch (jsonErr) {
      // Not JSON — try line format
    }

    // Line format: username|refresh_token
    const lines = trimmed.split('\n').filter(l => l.trim());
    const entries: BulkTokenEntry[] = [];
    for (const line of lines) {
      const parts = line.trim().split('|');
      if (parts.length !== 2) {
        throw new Error(`Invalid line format: "${line.trim()}". Expected: username|refresh_token`);
      }
      entries.push({ username: parts[0].trim(), refresh_token: parts[1].trim() });
    }
    return entries;
  }, []);

  const handleImport = useCallback(async () => {
    setError(null);
    setResponse(null);
    setRevealedKeys(new Set());

    let entries: BulkTokenEntry[];
    try {
      entries = parseInput(inputText);
    } catch (e: any) {
      setError(e.message);
      return;
    }

    if (entries.length === 0) {
      setError('No valid entries found');
      return;
    }

    setLoading(true);
    setProgress({ current: 0, total: entries.length });

    try {
      const result = await invoke<BulkTokenResponse>('bulk_import_tokens', { entries });
      setResponse(result);
    } catch (e: any) {
      setError(typeof e === 'string' ? e : e.message || 'Import failed');
    } finally {
      setLoading(false);
    }
  }, [inputText, parseInput]);

  const copyToClipboard = useCallback(async (text: string, index: number) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedIndex(index);
      setTimeout(() => setCopiedIndex(null), 2000);
    } catch { /* ignore */ }
  }, []);

  const toggleRevealKey = useCallback((index: number) => {
    setRevealedKeys(prev => {
      const next = new Set(prev);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
    });
  }, []);

  const exportResults = useCallback(() => {
    if (!response) return;
    const csv = [
      'Username,Email,API Key,Account ID,Status,Error',
      ...response.results.map(r =>
        [r.username, r.email || '', r.api_key || '', r.account_id || '', r.status, r.error || '']
          .map(f => `"${f}"`)
          .join(',')
      ),
    ].join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `bulk-tokens-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }, [response]);

  const exportJSON = useCallback(() => {
    if (!response) return;
    const blob = new Blob([JSON.stringify(response, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `bulk-tokens-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }, [response]);

  const maskKey = (key: string) => {
    if (key.length <= 10) return '••••••••••';
    return key.slice(0, 6) + '•'.repeat(key.length - 10) + key.slice(-4);
  };

  return (
    <div className="min-h-screen p-6 md:p-8">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
        >
          <h1 className="text-2xl font-bold text-base-content flex items-center gap-3">
            <div className="p-2 rounded-xl bg-gradient-to-br from-violet-500/20 to-purple-500/20">
              <Upload className="w-6 h-6 text-violet-500" />
            </div>
            {t('bulk_token.title', 'Bulk Token Import')}
          </h1>
          <p className="text-base-content/60 mt-1 text-sm">
            {t('bulk_token.subtitle', 'Import users from a list of refresh_tokens and generate proxy API keys (sk-...) for each user.')}
          </p>
        </motion.div>

        {/* Input Section */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="card bg-base-100 shadow-lg border border-base-300/50"
        >
          <div className="card-body">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <FileJson className="w-4 h-4 text-violet-500" />
                <span className="font-medium text-sm">{t('bulk_token.input_label', 'Input Data')}</span>
              </div>
              <button
                className="btn btn-xs btn-ghost text-base-content/50 hover:text-violet-500"
                onClick={() => setInputText(EXAMPLE_INPUT)}
              >
                {t('bulk_token.load_example', 'Load Example')}
              </button>
            </div>

            <textarea
              className="textarea textarea-bordered w-full h-48 font-mono text-xs leading-relaxed resize-y bg-base-200/50 focus:border-violet-500/50 focus:ring-1 focus:ring-violet-500/20"
              placeholder={`JSON format:\n${EXAMPLE_INPUT}\n\nOr line format:\nuser1|1//0abc...\nuser2|1//0def...`}
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              disabled={loading}
              id="bulk-token-input"
            />

            {/* Action Buttons */}
            <div className="flex items-center gap-3 mt-3">
              <button
                className="btn btn-primary gap-2 shadow-md"
                onClick={handleImport}
                disabled={loading || !inputText.trim()}
                id="bulk-token-import-btn"
              >
                {loading ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    {t('bulk_token.importing', 'Importing...')}
                  </>
                ) : (
                  <>
                    <Upload className="w-4 h-4" />
                    {t('bulk_token.import_btn', 'Start Import')}
                  </>
                )}
              </button>

              {inputText.trim() && !loading && (
                <button
                  className="btn btn-ghost btn-sm gap-1 text-base-content/50"
                  onClick={() => { setInputText(''); setResponse(null); setError(null); }}
                >
                  <Trash2 className="w-3.5 h-3.5" />
                  {t('bulk_token.clear', 'Clear')}
                </button>
              )}
            </div>
          </div>
        </motion.div>

        {/* Error Display */}
        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="alert alert-error shadow-md"
            >
              <AlertTriangle className="w-5 h-5" />
              <span className="text-sm">{error}</span>
              <button className="btn btn-ghost btn-xs" onClick={() => setError(null)}>✕</button>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Loading Progress */}
        <AnimatePresence>
          {loading && (
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
              className="card bg-base-100 shadow-lg border border-violet-500/30"
            >
              <div className="card-body py-6">
                <div className="flex items-center gap-4">
                  <div className="relative">
                    <Loader2 className="w-10 h-10 text-violet-500 animate-spin" />
                    <div className="absolute inset-0 flex items-center justify-center">
                      <div className="w-4 h-4 rounded-full bg-violet-500/20" />
                    </div>
                  </div>
                  <div className="flex-1">
                    <p className="font-medium text-base-content">
                      {t('bulk_token.processing', 'Processing entries...')}
                    </p>
                    <p className="text-xs text-base-content/50 mt-0.5">
                      {t('bulk_token.please_wait', 'This may take a while depending on the number of entries.')}
                    </p>
                  </div>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Results */}
        <AnimatePresence>
          {response && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4 }}
              className="space-y-4"
            >
              {/* Summary Stats */}
              <div className="grid grid-cols-3 gap-4">
                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: 0.1 }}
                  className="card bg-base-100 shadow border border-base-300/50"
                >
                  <div className="card-body py-4 px-5">
                    <p className="text-xs text-base-content/50 uppercase tracking-wide">{t('bulk_token.total', 'Total')}</p>
                    <p className="text-3xl font-bold bg-gradient-to-r from-violet-500 to-purple-500 bg-clip-text text-transparent">
                      {response.total}
                    </p>
                  </div>
                </motion.div>

                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: 0.2 }}
                  className="card bg-base-100 shadow border border-emerald-500/30"
                >
                  <div className="card-body py-4 px-5">
                    <p className="text-xs text-base-content/50 uppercase tracking-wide">{t('bulk_token.success', 'Success')}</p>
                    <p className="text-3xl font-bold text-emerald-500">
                      {response.success_count}
                    </p>
                  </div>
                </motion.div>

                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: 0.3 }}
                  className="card bg-base-100 shadow border border-red-500/30"
                >
                  <div className="card-body py-4 px-5">
                    <p className="text-xs text-base-content/50 uppercase tracking-wide">{t('bulk_token.failed', 'Failed')}</p>
                    <p className="text-3xl font-bold text-red-500">
                      {response.failed_count}
                    </p>
                  </div>
                </motion.div>
              </div>

              {/* Export Buttons */}
              <div className="flex items-center gap-2">
                <button
                  className="btn btn-sm btn-outline gap-1.5"
                  onClick={exportResults}
                  id="bulk-token-export-csv"
                >
                  <Download className="w-3.5 h-3.5" />
                  Export CSV
                </button>
                <button
                  className="btn btn-sm btn-outline gap-1.5"
                  onClick={exportJSON}
                  id="bulk-token-export-json"
                >
                  <FileJson className="w-3.5 h-3.5" />
                  Export JSON
                </button>
              </div>

              {/* Results Table */}
              <div className="card bg-base-100 shadow-lg border border-base-300/50 overflow-hidden">
                <div className="overflow-x-auto">
                  <table className="table table-sm">
                    <thead>
                      <tr className="bg-base-200/50">
                        <th className="text-xs font-medium">#</th>
                        <th className="text-xs font-medium">{t('bulk_token.col_username', 'Username')}</th>
                        <th className="text-xs font-medium">{t('bulk_token.col_email', 'Email')}</th>
                        <th className="text-xs font-medium">{t('bulk_token.col_api_key', 'API Key')}</th>
                        <th className="text-xs font-medium">{t('bulk_token.col_status', 'Status')}</th>
                        <th className="text-xs font-medium">{t('bulk_token.col_error', 'Error')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {response.results.map((result, idx) => (
                        <motion.tr
                          key={idx}
                          initial={{ opacity: 0, x: -10 }}
                          animate={{ opacity: 1, x: 0 }}
                          transition={{ delay: 0.05 * idx }}
                          className="hover:bg-base-200/30 transition-colors"
                        >
                          <td className="font-mono text-xs text-base-content/40">{idx + 1}</td>
                          <td className="font-medium text-sm">{result.username}</td>
                          <td className="text-xs text-base-content/60">{result.email || '—'}</td>
                          <td>
                            {result.api_key ? (
                              <div className="flex items-center gap-1.5">
                                <code className="text-xs font-mono bg-base-200 px-2 py-0.5 rounded select-all max-w-[200px] truncate">
                                  {revealedKeys.has(idx) ? result.api_key : maskKey(result.api_key)}
                                </code>
                                <button
                                  className="btn btn-ghost btn-xs p-0.5"
                                  onClick={() => toggleRevealKey(idx)}
                                  title={revealedKeys.has(idx) ? 'Hide' : 'Reveal'}
                                >
                                  {revealedKeys.has(idx) ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
                                </button>
                                <button
                                  className="btn btn-ghost btn-xs p-0.5"
                                  onClick={() => copyToClipboard(result.api_key!, idx)}
                                  title="Copy"
                                >
                                  {copiedIndex === idx ? (
                                    <CheckCircle2 className="w-3 h-3 text-emerald-500" />
                                  ) : (
                                    <Copy className="w-3 h-3" />
                                  )}
                                </button>
                              </div>
                            ) : (
                              <span className="text-xs text-base-content/30">—</span>
                            )}
                          </td>
                          <td>
                            {result.status === 'success' ? (
                              <span className="badge badge-sm gap-1 bg-emerald-500/10 text-emerald-600 border-emerald-500/20">
                                <CheckCircle2 className="w-3 h-3" /> OK
                              </span>
                            ) : (
                              <span className="badge badge-sm gap-1 bg-red-500/10 text-red-600 border-red-500/20">
                                <XCircle className="w-3 h-3" /> Fail
                              </span>
                            )}
                          </td>
                          <td className="text-xs text-red-500/70 max-w-[200px] truncate" title={result.error || ''}>
                            {result.error || ''}
                          </td>
                        </motion.tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

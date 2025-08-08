import { getConfig } from './config';

export interface ReplayFile {
  key: string;
  lastModified: Date;
  size: number;
  jobId?: string;
}

export class S3Service {
  private config = getConfig();

  /**
   * List all Arrow IPC replay files from S3
   */
  async listReplayFiles(): Promise<ReplayFile[]> {
    try {
      const response = await fetch(`${this.config.apiUrl}/api/v1/replays/list?_t=${Date.now()}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Cache-Control': 'no-cache, no-store, must-revalidate',
          'Pragma': 'no-cache',
          'Expires': '0',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to list files: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error listing replay files:', error);
      throw error;
    }
  }

  /**
   * Download Arrow IPC file from S3
   */
  async downloadReplayFile(key: string): Promise<ArrayBuffer> {
    try {
      const response = await fetch(`${this.config.apiUrl}/api/v1/replays/download?_t=${Date.now()}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Cache-Control': 'no-cache, no-store, must-revalidate',
          'Pragma': 'no-cache',
          'Expires': '0',
        },
        body: JSON.stringify({ key }),
      });

      if (!response.ok) {
        throw new Error(`Failed to download file: ${response.statusText}`);
      }

      return await response.arrayBuffer();
    } catch (error) {
      console.error('Error downloading replay file:', error);
      throw error;
    }
  }

  /**
   * Get specific replay file (using current static key structure)
   */
  async getCurrentReplay(): Promise<ArrayBuffer> {
    // Using the current static key from the compiler
    return this.downloadReplayFile('replays/job-123.arrow');
  }
}

export const s3Service = new S3Service();
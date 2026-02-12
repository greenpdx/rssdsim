import axios from 'axios'
import type { ModelInfo, LayoutResult } from '@/types'

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
})

export const modelApi = {
  async listModels(): Promise<ModelInfo[]> {
    const response = await api.get<ModelInfo[]>('/models')
    return response.data
  },

  async getModel(id: string): Promise<ModelInfo> {
    const response = await api.get<ModelInfo>(`/models/${id}/`)
    return response.data
  },

  async uploadModel(file: File): Promise<ModelInfo> {
    const formData = new FormData()
    formData.append('file', file)

    const response = await api.post<ModelInfo>('/models', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })
    return response.data
  },

  async getModelStructure(id: string): Promise<LayoutResult> {
    const response = await api.get<LayoutResult>(`/models/${id}/structure`)
    return response.data
  },

  async deleteModel(id: string): Promise<void> {
    await api.delete(`/models/${id}/`)
  },
}

export default api

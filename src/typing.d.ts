declare namespace Multipaste {
  type RecordType = 'text' | 'image'

  interface ClipboardRecord {
    id: number
    record_type: RecordType
    record_value: string
    pinned: boolean
  }
}

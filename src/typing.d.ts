declare namespace Multipaste {
  type RecordType = 'text' | 'image'

  interface ClipboardRecord {
    id: number
    record_type: RecordType
    record_value: string
    pinned: boolean
  }

  interface Config {
    auto_start: boolean
    max_items: number
  }
}

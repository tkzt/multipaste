declare namespace MultipasteHandler {
  type RecordType = 'text' | 'image'

  interface ClipboardRecord {
    id: number
    record_type: RecordType
  }
}

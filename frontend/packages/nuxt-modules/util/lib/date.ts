const pad2 = (num: string | number) => {
  return num.toString().padStart(2, '0')
}

export const dateString = (date?: Date) => {
  const d = date ?? new Date()
  const year = d.getFullYear()
  const month = d.getMonth() + 1
  const day = d.getDate()
  return `${year}-${pad2(month)}-${pad2(day)}`
}

export const timeString = (date?: Date) => {
  const d = date ?? new Date()
  const hours = d.getHours()
  const minutes = d.getMinutes()
  const seconds = d.getSeconds()
  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}`
}

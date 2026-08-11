import { describe, expect, it } from 'vitest'
import { decimalToMinor, minorToDecimal, projectEconomics } from './finance'
import type { RecordData } from './model'

const record = (entity: RecordData['entity'], data: Record<string, unknown>): RecordData => ({ id: Math.random().toString(), entity, createdAt: '', updatedAt: '', ...data }) as RecordData

describe('Finance and Outcome calculations', () => {
  it('converts money without JavaScript floating-point arithmetic', () => {
    expect(decimalToMinor('1000.25', 'CNY')).toBe('100025')
    expect(minorToDecimal('100025', 'CNY')).toBe('1000.25')
    expect(decimalToMinor('1200', 'JPY')).toBe('1200')
    expect(decimalToMinor('1.999', 'CNY')).toBe('1.999')
  })

  it('separates transfers from income and expense', () => {
    const records = [
      record('financialTransactions', { status: 'POSTED', transactionType: 'INCOME', baseAmountMinor: '10000000', projectId: 'p1' }),
      record('financialTransactions', { status: 'POSTED', transactionType: 'EXPENSE', baseAmountMinor: '6000000', projectId: 'p1' }),
      record('financialTransactions', { status: 'POSTED', transactionType: 'TRANSFER', baseAmountMinor: '2000000', projectId: 'p1' }),
      record('timeLogs', { projectId: 'p1', durationMinutes: 7200 }),
      record('results', { projectId: 'p1', actualAmountMinor: '4000000', evidenceStatus: 'VERIFIED' }),
    ]
    const result = projectEconomics(records, 'p1')
    expect(result.managementContributionMinor).toBe(4000000n)
    expect(result.cashNetMinor).toBe(4000000n)
    expect(result.unitTimeContributionMinor).toBe(33333n)
    expect(result.dataCoverage).toBe(100)
  })

  it('does not treat missing data as a complete zero result', () => {
    const result = projectEconomics([], 'p1')
    expect(result.incomeMinor).toBe(0n)
    expect(result.dataCoverage).toBe(0)
  })
})

import { useState, useEffect, useMemo } from 'react'
import { Box, Typography, Tabs, Tab } from '@mui/material'
import { DataGrid, GridColDef, GridToolbar } from '@mui/x-data-grid'
import * as XLSX from 'xlsx'

export interface ExcelViewerProps {
  binaryContent: Uint8Array
  path: string
}

/**
 * Excel viewer with multi-sheet support and sortable tables
 */
export function ExcelViewer({ binaryContent, path }: ExcelViewerProps) {
  const [workbook, setWorkbook] = useState<XLSX.WorkBook | null>(null)
  const [activeSheet, setActiveSheet] = useState<string>('')
  const [rows, setRows] = useState<any[]>([])
  const [columns, setColumns] = useState<GridColDef[]>([])
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  // Load workbook
  useEffect(() => {
    try {
      setLoading(true)
      setError(null)

      const wb = XLSX.read(binaryContent, { type: 'array' })
      setWorkbook(wb)

      if (wb.SheetNames.length > 0) {
        setActiveSheet(wb.SheetNames[0])
      } else {
        setError('No sheets found in workbook')
      }
    } catch (err) {
      console.error('Failed to load Excel file:', err)
      setError(err instanceof Error ? err.message : 'Failed to load Excel file')
    } finally {
      setLoading(false)
    }
  }, [binaryContent])

  // Parse active sheet
  useEffect(() => {
    if (!workbook || !activeSheet) {
      return
    }

    try {
      const sheet = workbook.Sheets[activeSheet]
      if (!sheet) {
        setError(`Sheet "${activeSheet}" not found`)
        return
      }

      // Convert sheet to JSON
      const jsonData = XLSX.utils.sheet_to_json(sheet, {
        header: 1,
        defval: '',
      }) as any[][]

      if (jsonData.length === 0) {
        setRows([])
        setColumns([])
        return
      }

      // First row as headers
      const headers = jsonData[0] as string[]
      const dataRows = jsonData.slice(1)

      // Create columns
      const cols: GridColDef[] = headers.map((header, index) => ({
        field: `col${index}`,
        headerName: header?.toString() || `Column ${index + 1}`,
        flex: 1,
        minWidth: 120,
        editable: false,
      }))
      setColumns(cols)

      // Create rows with IDs
      const rowsWithIds = dataRows.map((row, rowIndex) => {
        const rowData: any = { id: rowIndex }
        headers.forEach((_, colIndex) => {
          rowData[`col${colIndex}`] = row[colIndex] ?? ''
        })
        return rowData
      })
      setRows(rowsWithIds)
    } catch (err) {
      console.error('Failed to parse sheet:', err)
      setError(err instanceof Error ? err.message : 'Failed to parse sheet')
    }
  }, [workbook, activeSheet])

  const handleSheetChange = (_: React.SyntheticEvent, newValue: string) => {
    setActiveSheet(newValue)
  }

  const filename = path.split('/').pop() || 'workbook.xlsx'
  const sheetNames = workbook?.SheetNames || []
  const rowCount = useMemo(() => rows.length, [rows])
  const columnCount = useMemo(() => columns.length, [columns])

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box
        sx={{
          px: 2,
          py: 1,
          borderBottom: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>
            {filename}
          </Typography>
          {rowCount > 0 && (
            <Typography variant="caption" sx={{ color: 'text.secondary' }}>
              {rowCount.toLocaleString()} rows × {columnCount} columns
            </Typography>
          )}
        </Box>
      </Box>

      {/* Sheet Tabs */}
      {sheetNames.length > 1 && (
        <Box
          sx={{
            borderBottom: 1,
            borderColor: 'divider',
            bgcolor: 'background.paper',
          }}
        >
          <Tabs
            value={activeSheet}
            onChange={handleSheetChange}
            variant="scrollable"
            scrollButtons="auto"
            sx={{
              minHeight: 36,
              '& .MuiTab-root': {
                minHeight: 36,
                py: 0.5,
                textTransform: 'none',
              },
            }}
          >
            {sheetNames.map((name) => (
              <Tab key={name} label={name} value={name} />
            ))}
          </Tabs>
        </Box>
      )}

      {/* Data Grid */}
      <Box sx={{ flexGrow: 1, overflow: 'hidden' }}>
        {loading ? (
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
            }}
          >
            <Typography variant="body2" color="text.secondary">
              Loading Excel file...
            </Typography>
          </Box>
        ) : error ? (
          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              gap: 1,
            }}
          >
            <Typography variant="body1" color="error">
              Failed to load Excel file
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {error}
            </Typography>
          </Box>
        ) : rows.length === 0 ? (
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
            }}
          >
            <Typography variant="body2" color="text.secondary">
              Sheet is empty
            </Typography>
          </Box>
        ) : (
          <DataGrid
            rows={rows}
            columns={columns}
            slots={{ toolbar: GridToolbar }}
            slotProps={{
              toolbar: {
                showQuickFilter: true,
              },
            }}
            density="compact"
            disableRowSelectionOnClick
            sx={{
              border: 'none',
              '& .MuiDataGrid-columnHeaders': {
                bgcolor: 'action.hover',
              },
            }}
          />
        )}
      </Box>
    </Box>
  )
}

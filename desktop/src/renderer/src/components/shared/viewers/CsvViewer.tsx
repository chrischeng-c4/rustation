import { useState, useEffect, useMemo } from 'react'
import { Box, Typography } from '@mui/material'
import { DataGrid, GridColDef, GridToolbar } from '@mui/x-data-grid'
import Papa from 'papaparse'

export interface CsvViewerProps {
  content: string
  path: string
}

/**
 * CSV viewer with sortable table and search
 */
export function CsvViewer({ content, path }: CsvViewerProps) {
  const [rows, setRows] = useState<any[]>([])
  const [columns, setColumns] = useState<GridColDef[]>([])
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    try {
      setLoading(true)
      setError(null)

      // Parse CSV
      const result = Papa.parse(content, {
        header: true,
        dynamicTyping: true,
        skipEmptyLines: true,
      })

      if (result.errors.length > 0) {
        console.warn('CSV parsing warnings:', result.errors)
      }

      // Create columns from header
      if (result.data.length > 0) {
        const firstRow = result.data[0] as Record<string, any>
        const cols: GridColDef[] = Object.keys(firstRow).map((field) => ({
          field,
          headerName: field,
          flex: 1,
          minWidth: 150,
          editable: false,
        }))
        setColumns(cols)

        // Add IDs to rows (required by DataGrid)
        const rowsWithIds = result.data.map((row, index) => ({
          id: index,
          ...(row as Record<string, any>),
        }))
        setRows(rowsWithIds)
      }
    } catch (err) {
      console.error('Failed to parse CSV:', err)
      setError(err instanceof Error ? err.message : 'Failed to parse CSV')
    } finally {
      setLoading(false)
    }
  }, [content])

  const filename = path.split('/').pop() || 'data.csv'

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
              Loading CSV...
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
              Failed to load CSV
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
              No data found
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

import { createTheme, alpha } from '@mui/material/styles'

// Material Design 3 inspired theme
export const theme = createTheme({
  palette: {
    mode: 'dark', // Default to dark for developers
    primary: {
      main: '#D0BCFF', // M3 Dark Primary
      light: '#E8DEF8',
      dark: '#381E72',
      contrastText: '#381E72',
    },
    secondary: {
      main: '#CCC2DC',
      light: '#E8DEF8',
      dark: '#332D41',
      contrastText: '#332D41',
    },
    background: {
      default: '#1C1B1F', // M3 Dark Surface
      paper: '#2B2930',   // M3 Dark Surface Container
    },
    error: {
      main: '#F2B8B5',
    },
    text: {
      primary: '#E6E1E5',
      secondary: '#CAC4D0',
    },
    divider: alpha('#CAC4D0', 0.12),
  },
  shape: {
    borderRadius: 16, // M3 Medium Rounding
  },
  typography: {
    fontFamily: [
      'Inter',
      'ui-sans-serif',
      'system-ui',
      '-apple-system',
      'BlinkMacSystemFont',
      '"Segoe UI"',
      'Roboto',
      '"Helvetica Neue"',
      'Arial',
      'sans-serif',
    ].join(','),
    h1: { fontSize: '2.5rem', fontWeight: 600 },
    h2: { fontSize: '2rem', fontWeight: 600 },
    h3: { fontSize: '1.5rem', fontWeight: 600 },
    body1: { fontSize: '0.875rem', lineHeight: 1.5 },
    body2: { fontSize: '0.75rem', lineHeight: 1.43 },
    button: { textTransform: 'none', fontWeight: 500 },
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 20, // M3 Fully rounded buttons
          padding: '8px 24px',
        },
        containedPrimary: {
          backgroundColor: '#D0BCFF',
          color: '#381E72',
          '&:hover': {
            backgroundColor: '#E8DEF8',
          },
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          backgroundImage: 'none',
          backgroundColor: '#2B2930', // Surface Container
          borderRadius: 12,
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundImage: 'none',
        },
      },
    },
  },
})

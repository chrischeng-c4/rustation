import React from 'react'
import {
  Button,
  TextField as Input,
  Card,
  CardHeader,
  CardContent,
  CardActions as CardFooter,
  Chip as Badge,
  Divider as Separator,
  Box,
  Typography,
  Checkbox,
  Switch,
  LinearProgress as Progress,
  Select,
  MenuItem as SelectItem,
  Alert,
  AlertTitle,
  Accordion,
  AccordionSummary as AccordionTrigger,
  AccordionDetails as AccordionContent,
  Stack,
} from '@mui/material'
import {
  ErrorOutline as AlertCircle,
  Terminal,
  Info,
  CheckCircle as CheckCircle2,
  ExpandMore as ExpandMoreIcon
} from '@mui/icons-material'

// Internal Wrapper for Accordion to match shadcn-like API structure
const MuiAccordionItem = ({ children, ...props }: any) => <Accordion variant="outlined" {...props}>{children}</Accordion>
const MuiCardTitle = ({ children, ...props }: any) => <Typography variant="h6" {...props}>{children}</Typography>
const MuiCardDescription = ({ children, ...props }: any) => <Typography variant="body2" color="text.secondary" {...props}>{children}</Typography>

// Map string types to React components
export const A2UI_REGISTRY: Record<string, React.ComponentType<any> | string> = {
  // Primitives
  'div': Box,
  'span': 'span',
  'p': (props: any) => <Typography variant="body2" {...props} />,
  'h1': (props: any) => <Typography variant="h4" {...props} />,
  'h2': (props: any) => <Typography variant="h5" {...props} />,
  'h3': (props: any) => <Typography variant="h6" {...props} />,
  
  // UI Components
  'button': (props: any) => <Button variant={props.variant === 'outline' ? 'outlined' : 'contained'} {...props} />,
  'input': (props: any) => <Input size="small" variant="outlined" {...props} />,
  'label': (props: any) => <Typography variant="caption" fontWeight={600} {...props} />,
  'checkbox': Checkbox,
  'switch': Switch,
  'progress': (props: any) => <Progress variant="determinate" {...props} />,
  'badge': Badge,
  'separator': Separator,
  'scroll-area': (props: any) => <Box sx={{ overflow: 'auto' }} {...props} />,
  
  // Select Composite
  'select': Select,
  'select-trigger': Box, // Placeholder
  'select-value': Box,   // Placeholder
  'select-content': Box, // Placeholder
  'select-item': SelectItem,
  'select-group': Box,
  'select-label': Box,
  'select-separator': Separator,

  // Accordion Composite
  'accordion': Box,
  'accordion-item': MuiAccordionItem,
  'accordion-trigger': (props: any) => (
    <AccordionTrigger expandIcon={<ExpandMoreIcon />}>
      <Typography variant="subtitle2">{props.children}</Typography>
    </AccordionTrigger>
  ),
  'accordion-content': AccordionContent,

  // Card Composite
  'card': (props: any) => <Card variant="outlined" {...props} />,
  'card-header': (props: any) => <Box sx={{ p: 2, borderBottom: 1, borderColor: 'outlineVariant' }} {...props} />,
  'card-title': MuiCardTitle,
  'card-description': MuiCardDescription,
  'card-content': (props: any) => <CardContent {...props} />,
  'card-footer': CardFooter,
  
  // Alert Composite
  'alert': (props: any) => <Alert severity="info" variant="outlined" {...props} />,
  'alert-title': AlertTitle,
  'alert-description': (props: any) => <Typography variant="body2" {...props} />,

  // Icons
  'icon-alert': AlertCircle,
  'icon-terminal': Terminal,
  'icon-info': Info,
  'icon-check': CheckCircle2,
}

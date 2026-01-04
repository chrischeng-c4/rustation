import React from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Switch } from '@/components/ui/switch'
import { Progress } from '@/components/ui/progress'
import {
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
} from '@/components/ui/select'
import {
  Accordion,
  AccordionItem,
  AccordionTrigger,
  AccordionContent,
} from '@/components/ui/accordion'
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert'
import { AlertCircle, Terminal, Info, CheckCircle2 } from 'lucide-react'

// Map string types to React components
export const A2UI_REGISTRY: Record<string, React.ComponentType<any> | string> = {
  // Primitives
  'div': 'div',
  'span': 'span',
  'p': 'p',
  'h1': 'h1',
  'h2': 'h2',
  'h3': 'h3',
  
  // UI Components
  'button': Button,
  'input': Input,
  'label': Label,
  'checkbox': Checkbox,
  'switch': Switch,
  'progress': Progress,
  'badge': Badge,
  'separator': Separator,
  'scroll-area': ScrollArea,
  
  // Select Composite
  'select': Select,
  'select-trigger': SelectTrigger,
  'select-value': SelectValue,
  'select-content': SelectContent,
  'select-item': SelectItem,
  'select-group': SelectGroup,
  'select-label': SelectLabel,
  'select-separator': SelectSeparator,

  // Accordion Composite
  'accordion': Accordion,
  'accordion-item': AccordionItem,
  'accordion-trigger': AccordionTrigger,
  'accordion-content': AccordionContent,

  // Card Composite
  'card': Card,
  'card-header': CardHeader,
  'card-title': CardTitle,
  'card-description': CardDescription,
  'card-content': CardContent,
  'card-footer': CardFooter,
  
  // Alert Composite
  'alert': Alert,
  'alert-title': AlertTitle,
  'alert-description': AlertDescription,

  // Icons (mapped as special components)
  'icon-alert': AlertCircle,
  'icon-terminal': Terminal,
  'icon-info': Info,
  'icon-check': CheckCircle2,
}

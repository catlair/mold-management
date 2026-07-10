import * as React from "react"

import { cn } from "@/lib/utils"
import { Input } from "@/components/ui/input"

export interface InputNumberProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type" | "onChange"> {
  min?: number
  max?: number
  step?: number
  value?: number
  onChange?: (value: number | undefined) => void
}

const InputNumber = React.forwardRef<HTMLInputElement, InputNumberProps>(
  ({ className, min, max, step, value, onChange, ...props }, ref) => {
    const handleChange = React.useCallback(
      (e: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = e.target.value
        
        if (newValue === "") {
          onChange?.(undefined)
          return
        }
        
        const numValue = parseFloat(newValue)
        if (!isNaN(numValue)) {
          // Apply min/max constraints
          let constrainedValue = numValue
          if (min !== undefined && numValue < min) {
            constrainedValue = min
          }
          if (max !== undefined && numValue > max) {
            constrainedValue = max
          }
          onChange?.(constrainedValue)
        }
      },
      [min, max, onChange]
    )

    return (
      <Input
        type="number"
        ref={ref}
        className={cn("tabular-nums", className)}
        value={value ?? ""}
        onChange={handleChange}
        min={min}
        max={max}
        step={step}
        {...props}
      />
    )
  }
)
InputNumber.displayName = "InputNumber"

export { InputNumber }
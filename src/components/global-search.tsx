import * as React from "react"
import { useNavigate } from "react-router-dom"
import { CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "cmdk"
import { Search } from "lucide-react"

export interface SearchResult {
  id: string
  title: string
  description?: string
  type: string
  url: string
}

export interface SearchGroup {
  name: string
  items: SearchResult[]
}

const GlobalSearch: React.FC = () => {
  const [open, setOpen] = React.useState(false)
  const [searchQuery, setSearchQuery] = React.useState("")
  const [results, setResults] = React.useState<SearchGroup[]>([])
  const [loading, setLoading] = React.useState(false)
  const navigate = useNavigate()

  // Load all info tables via API
  const loadData = React.useCallback(async () => {
    setLoading(true)
    try {
      // This is a mock implementation - replace with actual API calls
      // For a real app, you would have 7 different API endpoints
      const mockData: SearchGroup[] = [
        {
          name: "Molds",
          items: [
            {
              id: "1",
              title: "Mold #1234",
              description: "Injection mold for plastic parts",
              type: "mold",
              url: "/molds/1234",
            },
          ],
        },
        {
          name: "Customers",
          items: [
            {
              id: "1",
              title: "Acme Corp",
              description: "Manufacturing company",
              type: "customer",
              url: "/customers/1",
            },
          ],
        },
        {
          name: "Products",
          items: [
            {
              id: "1",
              title: "Widget A",
              description: "Standard widget",
              type: "product",
              url: "/products/1",
            },
          ],
        },
        {
          name: "Work Orders",
          items: [
            {
              id: "1",
              title: "WO-2024-001",
              description: "Mold maintenance",
              type: "workOrder",
              url: "/work-orders/1",
            },
          ],
        },
        {
          name: "Maintenance",
          items: [
            {
              id: "1",
              title: "Preventive Maintenance",
              description: "Scheduled maintenance",
              type: "maintenance",
              url: "/maintenance/1",
            },
          ],
        },
        {
          name: "Documents",
          items: [
            {
              id: "1",
              title: "User Manual",
              description: "Mold operation guide",
              type: "document",
              url: "/documents/1",
            },
          ],
        },
        {
          name: "Users",
          items: [
            {
              id: "1",
              title: "John Doe",
              description: "Operator",
              type: "user",
              url: "/users/1",
            },
          ],
        },
      ]
      setResults(mockData)
    } catch (error) {
      console.error("Failed to load search data:", error)
      setResults([])
    } finally {
      setLoading(false)
    }
  }, [])

  // Debounced search
  React.useEffect(() => {
    const timeoutId = setTimeout(() => {
      loadData()
    }, 300)

    return () => clearTimeout(timeoutId)
  }, [loadData])

  // Keyboard shortcut to open search
  React.useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setOpen((open) => !open)
      }
    }

    document.addEventListener("keydown", down)
    return () => document.removeEventListener("keydown", down)
  }, [])

  // Filter results based on search query
  const filteredResults = React.useMemo(() => {
    if (!searchQuery) return results

    const query = searchQuery.toLowerCase()
    return results
      .map((group) => ({
        ...group,
        items: group.items.filter(
          (item) =>
            item.title.toLowerCase().includes(query) ||
            item.description?.toLowerCase().includes(query)
        ),
      }))
      .filter((group) => group.items.length > 0)
  }, [results, searchQuery])

  const handleSelect = (result: SearchResult) => {
    setOpen(false)
    navigate(result.url)
  }

  return (
    <>
      <button
        onClick={() => setOpen(true)}
        className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
      >
        <Search className="mr-2 h-4 w-4" />
        <span>Search...</span>
        <kbd className="pointer-events-none ml-2 hidden h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 sm:flex">
          <span className="text-xs">⌘</span>K
        </kbd>
      </button>
      <CommandDialog open={open} onOpenChange={setOpen}>
        <CommandInput
          placeholder="Search..."
          value={searchQuery}
          onValueChange={setSearchQuery}
        />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          {loading ? (
            <CommandGroup heading="Loading...">
              <CommandItem disabled>Loading search data...</CommandItem>
            </CommandGroup>
          ) : (
            filteredResults.map((group) => (
              <CommandGroup key={group.name} heading={group.name}>
                {group.items.map((result) => (
                  <CommandItem
                    key={result.id}
                    value={`${result.title} ${result.description || ""}`}
                    onSelect={() => handleSelect(result)}
                  >
                    <Search className="mr-2 h-4 w-4" />
                    <div className="flex flex-col">
                      <span>{result.title}</span>
                      {result.description && (
                        <span className="text-sm text-muted-foreground">
                          {result.description}
                        </span>
                      )}
                    </div>
                  </CommandItem>
                ))}
              </CommandGroup>
            ))
          )}
        </CommandList>
      </CommandDialog>
    </>
  )
}

export default GlobalSearch
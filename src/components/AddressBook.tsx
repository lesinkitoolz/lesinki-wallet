import React, { useState, useEffect } from 'react';

interface Contact {
  id: string;
  name: string;
  address: string;
  notes?: string;
  tags: string[];
  createdAt: number;
  lastUsed?: number;
}

interface AddressBookProps {
  onSelectAddress?: (address: string) => void;
}

const AddressBook: React.FC<AddressBookProps> = ({ onSelectAddress }) => {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [isAdding, setIsAdding] = useState(false);
  const [editingContact, setEditingContact] = useState<Contact | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTag, setSelectedTag] = useState<string>('all');
  const [newContact, setNewContact] = useState({
    name: '',
    address: '',
    notes: '',
    tags: [] as string[]
  });

  useEffect(() => {
    loadContacts();
  }, []);

  const loadContacts = () => {
    const saved = localStorage.getItem('lesinki-address-book');
    if (saved) {
      try {
        const parsedContacts = JSON.parse(saved);
        setContacts(parsedContacts);
      } catch (error) {
        console.error('Failed to load contacts:', error);
      }
    }
  };

  const saveContacts = (updatedContacts: Contact[]) => {
    localStorage.setItem('lesinki-address-book', JSON.stringify(updatedContacts));
    setContacts(updatedContacts);
  };

  const addContact = () => {
    if (!newContact.name.trim() || !newContact.address.trim()) {
      alert('Please fill in name and address');
      return;
    }

    const contact: Contact = {
      id: Date.now().toString(),
      name: newContact.name.trim(),
      address: newContact.address.trim(),
      notes: newContact.notes.trim(),
      tags: newContact.tags,
      createdAt: Date.now()
    };

    const updatedContacts = [...contacts, contact];
    saveContacts(updatedContacts);

    setNewContact({ name: '', address: '', notes: '', tags: [] });
    setIsAdding(false);
  };

  const updateContact = () => {
    if (!editingContact) return;

    const updatedContacts = contacts.map(contact =>
      contact.id === editingContact.id ? editingContact : contact
    );
    saveContacts(updatedContacts);
    setEditingContact(null);
  };

  const deleteContact = (id: string) => {
    if (confirm('Are you sure you want to delete this contact?')) {
      const updatedContacts = contacts.filter(contact => contact.id !== id);
      saveContacts(updatedContacts);
    }
  };

  const updateLastUsed = (id: string) => {
    const updatedContacts = contacts.map(contact =>
      contact.id === id
        ? { ...contact, lastUsed: Date.now() }
        : contact
    );
    saveContacts(updatedContacts);
  };

  const handleSelectAddress = (address: string, contactId: string) => {
    updateLastUsed(contactId);
    onSelectAddress?.(address);
  };

  const filteredContacts = contacts.filter(contact => {
    const matchesSearch = contact.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         contact.address.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         contact.notes?.toLowerCase().includes(searchTerm.toLowerCase());

    const matchesTag = selectedTag === 'all' || contact.tags.includes(selectedTag);

    return matchesSearch && matchesTag;
  });

  const allTags = Array.from(new Set(contacts.flatMap((contact: Contact) => contact.tags)));

  const sortedContacts = filteredContacts.sort((a: Contact, b: Contact) => {
    // Sort by last used first, then by creation date
    if (a.lastUsed && b.lastUsed) {
      return b.lastUsed - a.lastUsed;
    }
    if (a.lastUsed) return -1;
    if (b.lastUsed) return 1;
    return b.createdAt - a.createdAt;
  });

  return (
    <div className="address-book">
      <div className="book-header">
        <h2>Address Book</h2>
        <button
          onClick={() => setIsAdding(true)}
          className="add-contact-btn"
        >
          + Add Contact
        </button>
      </div>

      <div className="book-controls">
        <input
          type="text"
          placeholder="Search contacts..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="search-input"
        />
        <label htmlFor="tag-filter" className="sr-only">Filter by Tag</label>
        <select
          id="tag-filter"
          value={selectedTag}
          onChange={(e) => setSelectedTag(e.target.value)}
          className="tag-filter"
        >
          <option value="all">All Tags</option>
          {allTags.map((tag: string) => (
            <option key={tag} value={tag}>{tag}</option>
          ))}
        </select>
      </div>

      {contacts.length === 0 ? (
        <div className="empty-state">
          <p>No contacts yet. Add your first contact to get started!</p>
        </div>
      ) : (
        <div className="contacts-grid">
          {sortedContacts.map(contact => (
            <div key={contact.id} className="contact-card">
              <div className="contact-header">
                <h3>{contact.name}</h3>
                <div className="contact-actions">
                  <button
                    onClick={() => setEditingContact(contact)}
                    className="edit-btn"
                  >
                    ✏️
                  </button>
                  <button
                    onClick={() => deleteContact(contact.id)}
                    className="delete-btn"
                  >
                    🗑️
                  </button>
                </div>
              </div>

              <div className="contact-address">
                <code>{contact.address.slice(0, 12)}...{contact.address.slice(-8)}</code>
                <button
                  onClick={() => navigator.clipboard.writeText(contact.address)}
                  className="copy-btn"
                  title="Copy address"
                >
                  📋
                </button>
              </div>

              {contact.notes && (
                <div className="contact-notes">
                  {contact.notes}
                </div>
              )}

              {contact.tags.length > 0 && (
                <div className="contact-tags">
                  {contact.tags.map(tag => (
                    <span key={tag} className="tag">{tag}</span>
                  ))}
                </div>
              )}

              <div className="contact-meta">
                {contact.lastUsed && (
                  <span className="last-used">
                    Last used: {new Date(contact.lastUsed).toLocaleDateString()}
                  </span>
                )}
                <span className="created-date">
                  Added: {new Date(contact.createdAt).toLocaleDateString()}
                </span>
              </div>

              {onSelectAddress && (
                <button
                  onClick={() => handleSelectAddress(contact.address, contact.id)}
                  className="select-address-btn"
                >
                  Use This Address
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {(isAdding || editingContact) && (
        <div className="contact-modal">
          <div className="modal-content">
            <h3>{editingContact ? 'Edit Contact' : 'Add New Contact'}</h3>

            <div className="form-group">
              <label>Name:</label>
              <input
                type="text"
                value={editingContact ? editingContact.name : newContact.name}
                onChange={(e) => {
                  if (editingContact) {
                    setEditingContact({ ...editingContact, name: e.target.value });
                  } else {
                    setNewContact({ ...newContact, name: e.target.value });
                  }
                }}
                placeholder="Contact name"
              />
            </div>

            <div className="form-group">
              <label>Address:</label>
              <input
                type="text"
                value={editingContact ? editingContact.address : newContact.address}
                onChange={(e) => {
                  if (editingContact) {
                    setEditingContact({ ...editingContact, address: e.target.value });
                  } else {
                    setNewContact({ ...newContact, address: e.target.value });
                  }
                }}
                placeholder="Solana address"
              />
            </div>

            <div className="form-group">
              <label>Notes (optional):</label>
              <textarea
                value={editingContact ? editingContact.notes || '' : newContact.notes}
                onChange={(e) => {
                  if (editingContact) {
                    setEditingContact({ ...editingContact, notes: e.target.value });
                  } else {
                    setNewContact({ ...newContact, notes: e.target.value });
                  }
                }}
                placeholder="Additional notes"
                rows={3}
              />
            </div>

            <div className="form-group">
              <label>Tags (comma-separated):</label>
              <input
                type="text"
                value={editingContact ? editingContact.tags.join(', ') : newContact.tags.join(', ')}
                onChange={(e) => {
                  const tags = e.target.value.split(',').map(tag => tag.trim()).filter(tag => tag);
                  if (editingContact) {
                    setEditingContact({ ...editingContact, tags });
                  } else {
                    setNewContact({ ...newContact, tags });
                  }
                }}
                placeholder="friend, exchange, defi"
              />
            </div>

            <div className="modal-actions">
              <button
                onClick={() => {
                  setIsAdding(false);
                  setEditingContact(null);
                  setNewContact({ name: '', address: '', notes: '', tags: [] });
                }}
                className="cancel-btn"
              >
                Cancel
              </button>
              <button
                onClick={editingContact ? updateContact : addContact}
                className="save-btn"
              >
                {editingContact ? 'Update' : 'Add'} Contact
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AddressBook;